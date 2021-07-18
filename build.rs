fn main() {
    windows::build! {
        Windows::Win32::Globalization::{CP_ACP, CP_UTF8, MultiByteToWideChar, WideCharToMultiByte},
        Windows::Win32::System::Threading::PROCESS_CREATION_FLAGS,
        Windows::Win32::UI::WindowsAndMessaging::MessageBoxW,
    };

    embed_resource::compile("gpg-restart-agent.exe.manifest.rc");
}
