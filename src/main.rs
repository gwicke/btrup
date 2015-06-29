#![feature(plugin,custom_derive)]
#![plugin(docopt_macros)]

use std::process::Command;
use std::io;

extern crate time;
extern crate docopt;
extern crate rustc_serialize;

static BACKUP_DIR: &'static str = "/.backup";

fn run_cmd(cmd: &mut Command) -> io::Result<String> {
    println!("{:?}", cmd);
    cmd.output()
        .and_then(|output| if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(io::Error::new(io::ErrorKind::Other,
                               String::from_utf8_lossy(&output.stderr).to_string()))
        })
}

fn list_snapshots<'a>() -> io::Result<Vec<String>> {
    match run_cmd(Command::new("ls").args(&["-t", BACKUP_DIR])) {
        Ok(listing) => {
            let mut list = listing.trim().split('\n')
                .map(|s| BACKUP_DIR.to_string() + "/" + s).collect::<Vec<String>>();
            list.reverse();
            Ok(list)
        },
        Err(e) => {
            println!("Error: {}\nCreating directory {}", e, BACKUP_DIR);
            try!(run_cmd(Command::new("mkdir").arg(BACKUP_DIR)));
            Ok(vec!())
        },
    }
}

fn send_one_snapshot (snapshot: &str, 
                      maybe_base_snapshot_path: Option<&str>,
                      dest_dir: &str) -> io::Result<String> {
    let base_snapshot_path = match maybe_base_snapshot_path {
        None => "".to_string(),
        Some(source) => " -p ".to_string() + source,
    };
    let sh_cmd = "btrfs send".to_string() + &base_snapshot_path
        + " " + snapshot
        + " | btrfs receive " + dest_dir;
    run_cmd(Command::new("sh").args(&["-c", &sh_cmd]))
}

fn send_snapshot (new_snapshot: &str, base_snapshots: &Vec<String>, 
                  dest_dir: &str) -> io::Result<String> {
    // First, try to use one of the existing snapshots
    for base_snapshot_path in base_snapshots {
        match send_one_snapshot(&new_snapshot, Some(&base_snapshot_path), dest_dir) {
            Ok(out) => {
                println!("Backup done: {}", out);
                return Ok(base_snapshot_path.to_string()) // done
            },
            Err(err) => println!("{}", err), // continue
        }
    }
    Err(io::Error::new(io::ErrorKind::Other, "Did not find matching snapshot!"))
}

fn prune_snapshots (snapshots: &Vec<String>) -> io::Result<()> {
    // prune all but the last snapshot
    for snapshot in snapshots {
        println!("Pruning: {}", snapshot);
        try!(run_cmd(Command::new("btrfs").args(&["sub","del", &snapshot])));
    }
    Ok(())
}

fn run_backup(args: &Args) -> io::Result<String> {
    let snapshots = try!(list_snapshots());
    // TODO: use actual host name
    let new_snapshot = BACKUP_DIR.to_string() + "/pad." 
        + &time::now().rfc3339().to_string();
    try!(run_cmd(Command::new("btrfs").args(&["sub", "snap", "-r", "/", &new_snapshot])));
    try!(send_snapshot(&new_snapshot, &snapshots, &args.arg_dest));
    if args.flag_prune {
        try!(prune_snapshots(&snapshots));
    }
    Ok("All done".to_string())
}

docopt!(Args derive debug, "
    Usage: 
      backup [options] <dest>
      backup -h | --help

    Options:
      -h, --help      Show this message.
      -p, --prune     Prune old local snapshots.
    ");

fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    run_backup(&args).unwrap();
}
