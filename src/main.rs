use clap::Parser;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::os::unix::fs::FileExt;
use std::path::Path;

const BUFSIZE: u64 = 1024 * 1024;

struct Descriptor {
    depth: usize,
    prev_depth: usize,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
struct Args {
    #[arg(short, long, help = "target file")]
    target: String,

    #[arg(short, long, default_value_t = 512 * 1024 * 1024, help="chunk size boundary for output in bytes. Defaults to 512MB.")]
    chunksize: u64,
    #[arg(short, long, help = "Directory to output chunks to.")]
    out_dir: String,
}

impl Descriptor {
    fn new() -> Self {
        Self {
            depth: 0,
            prev_depth: 0,
        }
    }

    fn increment(&mut self) {
        self.prev_depth = self.depth;
        self.depth += 1;
    }

    fn decrement(&mut self) {
        self.prev_depth = self.depth;
        self.depth -= 1;
    }

    fn equalize(&mut self) {
        self.prev_depth = self.depth;
    }

    fn object_end(&self) -> bool {
        self.depth == 0 && self.prev_depth > 0
    }

    fn zero(&mut self) {
        self.depth = 0;
        self.prev_depth = 0;
    }
}

fn create_file_or_exit(name: &str) -> std::fs::File {
    if Path::new(name).exists() {
        eprintln!("Output file '{name}' already exists. Exiting...");
        std::process::exit(1);
    }

    match std::fs::File::create(name) {
        Ok(file) => file,
        Err(x) => {
            eprintln!("Could not create output file '{name}': {x}");
            std::process::exit(1);
        }
    }
}

impl std::fmt::Display for Descriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Descriptor: {} -> {}", self.prev_depth, self.depth)
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let mut infile = std::fs::File::open(&args.target)?;

    let chunksize = args.chunksize;
    let mut chunk = 0;

    match std::fs::create_dir_all(&args.out_dir) {
        Ok(_) => (),
        Err(x) => {
            format!("Could not create output directory '{}': {x}", &args.out_dir);
            std::process::exit(1);
        }
    }

    let mut outfile_name = format!("./{out_dir}/chunk_{chunk}.json", out_dir = &args.out_dir);
    let mut outfile = create_file_or_exit(&outfile_name);
    outfile.write(b"[")?;

    let mut begin: u64 = 1;
    let mut buf: [u8; BUFSIZE as usize] = [0; BUFSIZE as usize];

    let mut desc = Descriptor::new();

    infile.seek(SeekFrom::Start(begin as u64))?;
    let mut last_chunk_begin = 0;
    let mut last_begin = 0;
    while begin != last_begin {
        last_begin = begin;
        let read = infile.read_at(&mut buf, begin)?;
        for len in 0..read {
            if len >= read as usize {
                break;
            }

            match buf[len] {
                b'{' => desc.increment(),
                b'}' => desc.decrement(),
                _ => desc.equalize(),
            }

            if desc.object_end() {
                outfile.write(&buf[0..len + 1 as usize])?;
                desc.zero();
                begin += len as u64 + 1;

                if begin - last_chunk_begin > chunksize && begin != last_begin {
                    last_chunk_begin = begin;
                    begin += 1;
                    chunk += 1;
                    outfile.write(b"\n]")?;
                    outfile_name =
                        format!("./{out_dir}/chunk_{chunk}.json", out_dir = &args.out_dir);
                    outfile = create_file_or_exit(&outfile_name);
                    outfile.write(b"[")?;
                }
                break;
            }
        }
    }
    outfile.write(b"\n]")?;
    outfile.sync_all()?;

    if outfile.metadata().expect("Could not get Metadata").len() == 3 {
        drop(outfile);
        let mut last_outfile = std::fs::File::open(&outfile_name)?;
        let mut content = Vec::new();
        last_outfile
            .read_to_end(&mut content)
            .expect("Could not read last file");
        if content == b"[\n]" {
            drop(last_outfile);
            // this is an empty file we created because i
            // couldn't be bothered to implement the proper
            // logic above
            match std::fs::remove_file(&outfile_name) {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("Could not delete empty file '{outfile_name}' during cleanup: {err}");
                }
            };
        }
    }
    Ok(())
}
