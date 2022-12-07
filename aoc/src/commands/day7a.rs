use std::{
    any, borrow::Borrow, cell::RefCell, collections::VecDeque, fmt::Display, fs::File,
    iter::Peekable, path::PathBuf, rc::Rc, str::FromStr, task::Context,
};

use clap::Parser;

use crate::utils::{slurp_file, ParseError};

use super::{CommandImpl, DynError};

#[derive(Parser, Debug)]
pub struct Day7a {
    #[clap(long, short)]
    input: PathBuf,
}

impl CommandImpl for Day7a {
    fn main(&self) -> Result<(), DynError> {
        let lines: Vec<String> = slurp_file(&self.input)?;
        let fs = FileSystem::from_bash_history(lines)?;

        // Find total size of each dir
        // visualization
        // let sizes = fs
        //     .dir_sizes()
        //     .into_iter()
        //     .filter(|(d, size)| *size <= 100_000)
        //     .map(|(d, size)| (d.as_ref().borrow().name().to_owned(), size))
        //     .collect::<Vec<_>>();
        let sizes = fs
            .dir_sizes()
            .into_iter()
            .filter(|(d, size)| *size <= 100_000)
            .map(|(d, size)| size)
            .sum::<usize>();
        eprintln!("Sizes: {:?}", sizes);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ShellContext {
    location: VecDeque<Rc<RefCell<Entity>>>,
}

impl ShellContext {
    fn new() -> Self {
        Self { location: VecDeque::new() }
    }

    fn push(&mut self, dir: Rc<RefCell<Entity>>) {
        self.location.push_front(dir);
    }

    fn pop(&mut self) -> Option<Rc<RefCell<Entity>>> {
        self.location.pop_front()
    }

    fn current(&self) -> Option<Rc<RefCell<Entity>>> {
        self.location.front().cloned()
    }

    fn current_mut(&mut self) -> Option<&mut Rc<RefCell<Entity>>> {
        self.location.front_mut()
    }
}

#[derive(Debug, Clone)]
pub enum Entity {
    File { size: usize, name: String },
    Dir { name: String, entities: Vec<Rc<RefCell<Entity>>>, prev: Option<Rc<RefCell<Entity>>> },
}

impl Entity {
    pub fn name(&self) -> &str {
        use Entity::*;
        match self {
            File { name, .. } => name,
            Dir { name, .. } => name,
        }
    }

    fn is_dir(&self) -> bool {
        use Entity::*;
        match self {
            File { .. } => false,
            Dir { .. } => true,
        }
    }
    fn is_file(&self) -> bool {
        use Entity::*;
        match self {
            File { .. } => true,
            Dir { .. } => false,
        }
    }
    /// Check if a given name exists within this entity.
    ///
    /// If entity is a file, this compares the names.
    /// If entity is a dir, it looks for entities within the dir that match that name.
    fn exists(&self, name: &str) -> bool {
        use Entity::*;
        match self {
            File { name: fname, .. } => fname == name,
            Dir { entities, .. } => {
                name == ".." || entities.iter().any(|e| e.as_ref().borrow().name() == name)
            }
        }
    }

    /// Get a reference to an entity whin a dir
    fn get(&self, name: &str) -> Option<Rc<RefCell<Entity>>> {
        use Entity::*;
        match self {
            File { .. } => None,
            Dir { entities, .. } => {
                entities.iter().find(|e| e.as_ref().borrow().name() == name).cloned()
            }
        }
    }

    fn size(&self) -> usize {
        match self {
            Entity::File { size, .. } => *size,
            Entity::Dir { entities, .. } => {
                entities.iter().map(|e| e.as_ref().borrow().size()).sum()
            }
        }
    }

    fn size_rec_start(&self) -> Vec<(Rc<RefCell<Entity>>, usize)> {
        let mut sizes = vec![];
        self.size_rec(&mut sizes);
        sizes
    }

    fn size_rec(&self, sizes: &mut Vec<(Rc<RefCell<Entity>>, usize)>) -> usize {
        match self {
            Entity::File { size, .. } => *size,
            Entity::Dir { entities, .. } => {
                let mut total = 0;
                for entity in entities {
                    let size = entity.as_ref().borrow().size_rec(sizes);
                    total += size;
                    if entity.as_ref().borrow().is_dir() {
                        sizes.push((entity.clone(), size));
                    }
                }
                total
            }
        }
    }

    fn from_str(s: &str, ctx: &ShellContext) -> Result<Self, ParseError> {
        if s.starts_with("dir") {
            // It's a dir!
            let name = s.strip_prefix("dir ").unwrap().to_owned();
            Ok(Entity::Dir { name, entities: vec![], prev: ctx.current() })
        } else {
            // It's a file!
            let (size, name) =
                s.split_once(' ').ok_or_else(|| ParseError::new(format!("Invalid file: `{s}`")))?;
            Ok(Entity::File {
                size: size
                    .parse::<usize>()
                    .map_err(|e| ParseError::new(format!("Invalid file size `{size}`: {}", e)))?,
                name: name.to_owned(),
            })
        }
    }

    /// Must be a dir
    fn push_entity(&mut self, entity: Rc<RefCell<Entity>>) {
        assert!(self.is_dir());
        match self {
            Entity::Dir { entities, .. } => entities.push(entity),
            _ => unreachable!(),
        }
    }

    /// Print the in a nice tree-like format
    fn pretty_print(&self, f: &mut std::fmt::Formatter<'_>, ident: usize) -> std::fmt::Result {
        // Pad with spaces - there must be a better way
        for _ in 0..ident {
            write!(f, " ")?;
        }
        match self {
            Entity::File { size, name } => writeln!(f, "- {name} (file, size={size})"),
            Entity::Dir { name, entities, .. } => {
                writeln!(f, "- {name} (dir)")?;
                for entity in entities {
                    entity.as_ref().borrow().pretty_print(f, ident + 2)?
                }
                Ok(())
            }
        }
    }
}

impl Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.pretty_print(f, 0)
    }
}

#[derive(Debug, Clone)]
pub struct FileSystem {
    root: Rc<RefCell<Entity>>,
}

impl FileSystem {
    /// Create a new [`FileSystem`] with a `/` dir.
    pub fn new() -> Self {
        let root = Entity::Dir { name: String::from("/"), entities: vec![], prev: None };
        Self { root: Rc::new(RefCell::new(root)) }
    }

    pub fn from_bash_history(history: Vec<String>) -> Result<Self, ParseError> {
        let mut fs = FileSystem::new();
        let mut ctx = ShellContext::new();

        let mut history_iter = history.into_iter().peekable();
        while let Some(line) = history_iter.next() {
            let command = Command::parse_from_stream(line, &mut history_iter)?;
            let _exit_code = command.exec(&mut fs, &mut ctx);
        }

        Ok(fs)
    }

    /// Get the sizes of each dir
    pub fn dir_sizes(&self) -> Vec<(Rc<RefCell<Entity>>, usize)> {
        // let mut sizes = vec![];
        // This is super inefficient and we should have a bottom up approach, but I'm sick of this
        // sizes.push((self.root.clone(), self.root.as_ref().borrow().size()));

        // for entry in self.root.as_ref().borrow() {}
        // sizes
        let sizes = self.root.as_ref().borrow().size_rec_start();
        sizes
    }

    /// Get size of root, it would be better to get root in the rec function, but whatever
    pub fn root_size(&self) -> usize {
        self.root.as_ref().borrow().size()
    }
}

impl Display for FileSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.root.as_ref().borrow())
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum ExitCode {
    Success = 0,
}

#[derive(Debug, Clone)]
enum Command {
    Cd { target: String },
    Ls { output: Vec<String> },
}

impl Command {
    fn exec(&self, fs: &mut FileSystem, ctx: &mut ShellContext) -> Result<ExitCode, ParseError> {
        use Command::*;
        match self {
            Cd { target } => {
                // Entity creation handled in ls
                // We can only cd into dirs already found by ls
                // All we have to do here is adjust the ctx

                if target == "/" && ctx.location.is_empty() {
                    ctx.push(fs.root.clone())
                } else {
                    // Check that the target exists in the current dir
                    if let Some(current) = ctx.current() {
                        if target == ".." {
                            _ = ctx.pop();
                        } else {
                            assert!(current.as_ref().borrow().exists(target));
                            let go_into = current.as_ref().borrow().get(target).unwrap();
                            ctx.push(go_into);
                        }
                    } else {
                        unreachable!();
                    }
                }
            }
            Ls { output } => {
                // Parse the output and add onto the file system based on where we currently are with ctx
                for line in output {
                    let entity = Entity::from_str(line, ctx)?;
                    let current = ctx.current_mut().expect("No context yet. error");
                    current.borrow_mut().push_entity(Rc::new(RefCell::new(entity)));
                }
            }
        }
        Ok(ExitCode::Success)
    }

    fn parse_from_stream<I>(
        command_line: String,
        stream: &mut Peekable<I>,
    ) -> Result<Self, ParseError>
    where
        I: Iterator<Item = String>,
    {
        let s = &command_line;
        let s = s
            .strip_prefix("$ ")
            .ok_or_else(|| ParseError::new(format!("Invalid command: `{s}`")))?;

        if s.starts_with("cd") {
            let s = s.strip_prefix("cd ").unwrap();
            Ok(Command::Cd { target: s.to_owned() })
        } else if s.starts_with("ls") {
            let mut output = vec![];
            let mut stop = false;
            loop {
                // Check whether the next line on the iterator is a command
                // Or if we've hit EOF
                if let Some(peeked_line) = stream.peek() {
                    if peeked_line.starts_with("$") {
                        break;
                    }
                } else {
                    break;
                }

                // Safe since we've peeked
                output.push(stream.next().unwrap());
            }

            Ok(Command::Ls { output })
        } else {
            unimplemented!()
        }
    }
}
