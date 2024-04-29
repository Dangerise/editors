use iced::widget::{button, column, container, horizontal_space, row, text, text_editor};
use iced::{executor, window, Application, Command, Length, Settings, Size, Theme};

use std::io;
use std::path::{Path, PathBuf};

fn main() {
    Editor::run(Settings {
        window: window::Settings {
            size: Size {
                width: 200.,
                height: 200.,
            },
            ..Default::default()
        },
        ..Default::default()
    })
    .unwrap();
}

#[derive(Debug)]
struct Editor {
    editor_content: text_editor::Content,
    current_file_path: Option<PathBuf>,
    error: Option<Error>,
}

#[derive(Debug, Clone)]
enum Message {
    Edit(text_editor::Action),
    OpenFile,
    OpenFileResult(Result<(PathBuf, String), Error>),
}

#[derive(Debug, Clone)]
enum Error {
    FileDialogCanceled,
    IO(io::ErrorKind),
}

impl Application for Editor {
    type Flags = ();
    type Message = Message;
    type Executor = executor::Default;
    type Theme = Theme;
    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            Self {
                editor_content: text_editor::Content::with_text(include_str!("main.rs")),
                current_file_path: None,
                error: None,
            },
            Command::none(),
        )
    }
    fn title(&self) -> String {
        String::from("iced editor")
    }
    fn view(&self) -> iced::Element<'_, Message, Theme, iced::Renderer> {
        let control_bar = {
            let open_file = button("Open").on_press(Message::OpenFile);
            row![open_file]
        };

        let text_editor = text_editor(&self.editor_content)
            .on_action(Message::Edit)
            .height(Length::enclose(Length::Fill, Length::Fixed(50.)));

        let status_bar = {
            let info = {
                let file_path = self
                    .current_file_path
                    .as_ref()
                    .map_or("New File".to_string(), |p| format!("{}", p.display()));

                let info = text(file_path);

                info
            };

            let (cursor_position_line, cursor_position_column) =
                self.editor_content.cursor_position();
            let cursor_position_text = text(format!(
                "{}:{}",
                cursor_position_line + 1,
                cursor_position_column + 1
            ));

            row![info, horizontal_space(), cursor_position_text]
        };

        container(column![control_bar, text_editor, status_bar])
            .padding(10)
            .into()
    }
    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Edit(action) => {
                self.editor_content.perform(action);
                Command::none()
            }
            Message::OpenFile => Command::perform(open_file(), Message::OpenFileResult),
            Message::OpenFileResult(Ok((path, file))) => {
                self.editor_content = text_editor::Content::with_text(&file);
                self.current_file_path = Some(path);
                Command::none()
            }
            Message::OpenFileResult(Err(err)) => {
                self.error = Some(err);
                Command::none()
            }
        }
    }
}

async fn open_file() -> Result<(PathBuf, String), Error> {
    let handle = rfd::AsyncFileDialog::new()
        .set_directory("./")
        .pick_file()
        .await
        .ok_or(Error::FileDialogCanceled)?;

    let path = handle.path();

    let file = load_file(path).await?;

    Ok((path.to_path_buf(), file))
}

async fn load_file(path: impl AsRef<Path>) -> Result<String, Error> {
    tokio::fs::read_to_string(path)
        .await
        .map_err(|e| Error::IO(e.kind()))
}
