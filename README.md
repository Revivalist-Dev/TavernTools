# Tools for working with SillyTavern cards
 
This tool supports both `chara_card_v2` and `chara_card_v3` formats for SillyTavern cards.
 
## Currently supported functions:
 
* `tavern_card_tools.exe print <filename.png>` - print the meaningful content of the character data to the terminal.
* `tavern_card_tools.exe <filename.png>` - same as above, print the character data.
* `tavern_card_tools.exe print_all <filename.png>` - print all character data as JSON to the terminal.
* `tavern_card_tools.exe print_json_file <filename.json>` - print the content of a JSON card file (supports both v2 and v3 formats).
* `tavern_card_tools.exe extract_json <filename.png> <output.json>` - extract the embedded JSON from a PNG card and save it to a specified `.json` file.
* `tavern_card_tools.exe extract_image <filename.png> <output.png>` - extract the image data from a PNG card (without embedded JSON) and save it to a new `.png` file.
* `tavern_card_tools.exe baya_get <URL>` - extract a character card from "Backyard AI" URL. Supports URLs that require registration. Will automatically convert all instances of word `User` into `{{user}}`
* `tavern_card_tools.exe de8 <filename.png>` - remove paired asterisks from all primary text fields of the card. Creates a new file for the output, named de8.filename.png, and leaves original as it is.
Add `--force` flag to overwrite output file even if it already exists.
* `tavern_card_tools.exe process_all` - processes all PNG cards in the default input directory, extracting JSON and image, and handling errors by moving problematic cards to appropriate issue subfolders.
 
## Default Paths
 
By default, the tool uses the following paths within the project's root directory:
 
*   **Input for character cards**: `inventory/input/`
*   **Output for processed cards**: `inventory/output/`
*   **Logs**: `inventory/last_run.log`
*   **Issue cards**: `inventory/issue/`
    *   Cards that fail due to format issues will be moved to `inventory/issue/format/`
    *   Cards that have no data will be moved to `inventory/issue/no_data/`
 
You can override these paths by explicitly providing them as arguments to the commands.
 
## Installation
 
Windows folks - download .EXE from [releases](https://github.com/Barafu/tavern_card_tools/releases/latest). No need to install, should just work.
 
Linux crowd - you better build it from source. Download this repository. Install `cargo` and `rustc` packages.
Type `cargo build --release` in the root folder of the repo. It will download dependencies and build.  Here is your app in `target/release` folder.
