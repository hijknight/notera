mod ui;
mod storage;
mod input;



use input::get_user_input;
use ui::NoteUI;


fn main() {
    loop {
        println!();
        println!("| 1. Create Note\n| 2. View Notes\n| 3. Edit Note\n| 4. Delete Note\n| q. Exit");
        println!();
        let choice = get_user_input("| Choose an option: ");

        match choice.trim() {
            "1" => {
                let title = get_user_input("Enter title: ");
                storage::save_note(&title);
            }
            "2" => {
                let notes = storage::read_notes();
                if let Err(e) = NoteUI::draw_ui(&notes) {
                    eprintln!("Error rendering UI: {}", e);
                }
            }
            "3" => {
                let title = get_user_input("Enter title of note to edit: ");
                storage::edit_note(&title);
            }
            "4" => {
                let title = get_user_input("Enter title of note to delete: ");
                storage::delete_note(&title);
            }
            "q" => break,
            _ => println!("Invalid choice, try again."),
        }
    }
}
