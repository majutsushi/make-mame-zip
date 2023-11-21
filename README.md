# Assemble individual game ZIPs from a ROM set for specific MAME versions

`make-mame-zip` is a simple tool that can create self-contained ZIP files
of individual MAME games that work with a specific MAME version,
provided that the available ROM sets contain all of the required ROMs.

Since different MAME versions often require different individual ROM files for certain games,
not all ROM sets work with all MAME versions.
In addition ROM sets can be in [different "merged" states](https://docs.mamedev.org/usingmame/aboutromsets.html),
meaning that not all game ZIPs are necessarily self-contained.
There are tools like [clrmamepro](https://mamedev.emulab.it/clrmamepro/) that are useful for organizing entire ROM set collections,
but if you are only interested in a handful of games then they require too much work to use.
`make-mame-zip` is meant to assemble individual,
self-contained games from available ROM sets regardless of their version and merged state.

## Usage

`make-mame-zip` has two subcommands: `create-db` and `make-zip`.

### Creating the ROM database

The `create-db` subcommand creates a database of all of the available ROMs,
their checksums,
and their locations.
Note that this means that if you move your ROM sets you will have to recreate the database.

Example usage:

```
$ make-mame-zip create-db /path/to/romset /path/to/rollback-romset
```

The location of the database depends on the operating system:

-   **Linux**: `$XDG_DATA_HOME` or `$HOME/.local/share` if `$XDG_DATA_HOME` is not set
-   **macOS**: `$HOME/Library/Application Support`
-   **Windows**: `{FOLDERID_LocalAppData}`, e.g. `C:\Users\<user>\AppData\Local`

### Creating a game ZIP

The command to create a ZIP file containing all of the ROM files needed to run a game
requires two arguments:
the path to a MAME XML DAT file and the canonical MAME name of the game.
You can create the right DAT file to use for your MAME version
by running `mame -listxml > mame.xml`.
Some frontends may also allow you to create such a DAT file without using the command line.

When running the command it will check the database for the ROM checksums listed in the DAT file
and create the game ZIP file by extracting the required ROMs from the relevant ROM sets.

Example usage:

```
$ make-mame-zip make-zip /path/to/mame.xml pacman
```

This will create a file `pacman.zip` that works with your MAME version.
