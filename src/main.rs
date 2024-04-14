use native_dialog::FileDialog;
use iced::{Element, Sandbox, Settings, Theme};
use iced::widget::{radio, scrollable, column, row, container, text_input, text, button};
use rust_search::{SearchBuilder, FilterExt};
use std::path::PathBuf;
use std::fs;

fn main() -> iced::Result {
	Hello::run(Settings::default())
}

struct Hello{
    prefix : String,
    file_paths: Vec<PathBuf>,
    logs : String,
    ignore_case: bool,
    file_ext: String,
    search_hidden_files: bool,
    exclude_dirs: bool,
    maybe_suffix: bool,
    new_file_paths: Vec<PathBuf>,
}


#[derive(Debug, Clone)]
enum Interaction {
    Rename,
    UndoRename,
	RefreshLogs,
    PickFolder,
    PrefixSubmitted(String),
    IgnoreCase(bool),
    FileExtAdded(String),
    SearchHidden(bool),
    ExcludeDirs(bool),
    RemoveSuffix(bool)

}


impl Sandbox for Hello {
    type Message = Interaction;
    
    fn new() -> Hello {
        Hello {
            prefix : "".to_string(),
            logs : "No logs generated yet".to_string(),
            file_paths : vec![PathBuf::from("./example/path/file")],
            ignore_case: false,
            file_ext: "".to_string(),
            search_hidden_files: false,
            exclude_dirs: false,
            maybe_suffix: false,
            new_file_paths: vec![]
        }
    } 

    fn title(&self) -> String {
        String::from("Annoying Prefix Remover")
    }

    fn update(&mut self, message: Self::Message) {

        match message {
        Interaction::Rename => {
            let mut log: Vec<String> = vec![];
            let mut nfp: Vec<PathBuf> = vec![];
            for p in &self.file_paths{
                let mut file_name: String = p.clone().file_name().unwrap().to_os_string().into_string().unwrap();
                let mut new_p = p.clone();
                
                //remove prefix or suffix
                if self.maybe_suffix{
                    file_name = file_name.chars().rev().collect();
                    let suffix: String = self.prefix.chars().rev().collect();
                    file_name = file_name.replacen(&suffix, "", 1);
                    file_name = file_name.chars().rev().collect();
                    new_p.set_file_name(&file_name);
                    fs::rename(&p, &new_p).unwrap();
                    log.push(file_name);
                }
                else{
                    file_name = file_name.replacen(&self.prefix, "", 1);
                    new_p.set_file_name(&file_name);
                    fs::rename(&p, &new_p).unwrap();
                    log.push(file_name);
                }
                nfp.push(new_p);
            }
            self.logs = log.join("\n");
            self.new_file_paths = nfp;
        }
        Interaction::UndoRename => {
            let mut i = 0;
            for path in &self.file_paths{
                fs::rename(&self.new_file_paths[i], path).unwrap();
                i += 1;
            }
            self.logs = self.file_paths.clone().into_iter().map(|p| p.file_name().unwrap().to_os_string().into_string().unwrap()).collect::<Vec<String>>().join("\n");
        }
		Interaction::RefreshLogs => {
            self.prefix = "".to_string();
            self.logs = "Cleared Logs".to_string();
            self.file_paths = vec![PathBuf::from("./example/path")];
            self.ignore_case = false;
            self.file_ext = "".to_string();
            self.search_hidden_files = false;
            self.exclude_dirs = false;
            self.maybe_suffix = false;
            self.new_file_paths = vec![];
 
        },
		Interaction::PickFolder  => {
            let path = FileDialog::new().show_open_single_dir().unwrap();
            match path {
                    Some(path) => {
                        if self.prefix == "".to_string(){
                            return
                        }
                        let mut search = SearchBuilder::default().location(path).search_input(&self.prefix);
                        if self.file_ext != "".to_string() {
                            search = search.ext(&self.file_ext);
                        }
                        if self.ignore_case {
                            search = search.ignore_case();
                        }
                        if self.search_hidden_files {
                            search = search.hidden();
                        }
                        if self.exclude_dirs {
                            search = search.custom_filter(|dir| !dir.metadata().unwrap().is_dir());
                        }

                        let search: Vec<String> = search.build().collect();
                        let s = &self.prefix;

                        self.file_paths = search.into_iter().map(|p| {PathBuf::from(p)}).collect();
                        if self.maybe_suffix {
                            self.file_paths = self.file_paths.clone().into_iter().filter(|p| {
                                if self.ignore_case { return p.file_stem().unwrap().to_os_string().into_string().unwrap().to_lowercase().ends_with(&s.to_lowercase())}
                                else { return p.file_stem().unwrap().to_os_string().into_string().unwrap().ends_with(s) }
                            }).collect();
                        }
                        else{
                            self.file_paths = self.file_paths.clone().into_iter().filter(|p| {
                                if self.ignore_case { return p.file_stem().unwrap().to_os_string().into_string().unwrap().to_lowercase().starts_with(&s.to_lowercase())}
                                else { return p.file_stem().unwrap().to_os_string().into_string().unwrap().ends_with(s) }
                            }).collect()
                        }
                        self.logs = self.file_paths.clone().into_iter().map(|p| {p.file_name().unwrap().to_os_string().into_string().unwrap()}).collect::<Vec<String>>().join("\n");
                    
                    },
                    None => {
                        self.logs = "Please select a Folder/Directory to start searching".to_string();
                    }
                }
            
            }
        }
        Interaction::PrefixSubmitted(pre) => {
            self.prefix = pre;
        }
        Interaction::IgnoreCase(choice) => {
            self.ignore_case = !choice;
        }
        Interaction::FileExtAdded(ext) => {
            self.file_ext = ext;
        }
        Interaction::SearchHidden(choice) => {
            self.search_hidden_files = !choice;
        }
        Interaction::ExcludeDirs(choice) => {
            self.exclude_dirs = !choice;
        }
        Interaction::RemoveSuffix(choice) =>{
            self.maybe_suffix = !choice;
        }
    }
    }

    fn view(&self) -> Element<Self::Message> {
        let log_text = text(&self.logs);
	    let refresh_btn = button("Refresh Logs").on_press(Interaction::RefreshLogs);
        let folder_picker_btn = button("Choose Folder to Search").on_press(Interaction::PickFolder);
        let start_rename_btn = button("Start Renaming").on_press(Interaction::Rename);
        let prefix_text_input = text_input("Enter Prefix to remove", &self.prefix).on_input(Interaction::PrefixSubmitted).padding(10);
        let ext_text_input = text_input("Search By Extention (Leave blank to search all types of files)", &self.file_ext).on_input(Interaction::FileExtAdded).padding(10);
        let ignore_case_radio = radio("Ignore Case?", self.ignore_case, Some(true), Interaction::IgnoreCase);
        let search_hidden_files_radio = radio("Search Hidden Files?", self.search_hidden_files, Some(true), Interaction::SearchHidden);
        let exclude_directories_radio = radio("Exclude Directories/Folders?", self.exclude_dirs, Some(true), Interaction::ExcludeDirs);
        let maybe_suffix_radio = radio("Remove Suffix instead of Prefix", self.maybe_suffix, Some(true), Interaction::RemoveSuffix);
        let undo_btn = button("Undo Rename").on_press(Interaction::UndoRename);
	    container(row![
                  column![prefix_text_input, folder_picker_btn, ignore_case_radio, search_hidden_files_radio, exclude_directories_radio, maybe_suffix_radio, ext_text_input, start_rename_btn, undo_btn], 
                  column![refresh_btn, scrollable(log_text)]
        ].spacing(10)
        ).padding(10).into()
    }

    fn theme(&self) -> Theme{
	    Theme::Dark
    }
}
