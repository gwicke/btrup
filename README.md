# btrup: Simple but fast btrfs-based backups in Rust

Implements https://btrfs.wiki.kernel.org/index.php/Incremental_Backup.

Requirements:
- root fs with btrfs
- usb disk with btrfs
- btrfs-tools

```
Usage: 
      btrup [options] <dest>
      btrup -h | --help

    Options:
      -h, --help      Show this message.
      -p, --prune     Prune old local snapshots.
    
    Example:
      sudo btrup -p /mnt/usbdisk/backup
```

