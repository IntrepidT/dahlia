use super::types::ConnectionStatus;
use leptos::prelude::*;

#[component]
pub fn ConnectionStatusIndicator(
    #[prop(into)] connection_status: Signal<ConnectionStatus>,
    #[prop(into)] error_message: Signal<Option<String>>,
) -> impl IntoView {
    // Combined class computation for the main status div
    let status_div_class = move || {
        let base_classes = "flex items-center space-x-2 px-3 py-1 rounded-full text-sm";
        match connection_status.get() {
            ConnectionStatus::Connected => format!("{} bg-green-100 text-green-800", base_classes),
            ConnectionStatus::Connecting => {
                format!("{} bg-yellow-100 text-yellow-800", base_classes)
            }
            ConnectionStatus::Error => format!("{} bg-red-100 text-red-800", base_classes),
            ConnectionStatus::Disconnected => format!("{} bg-gray-100 text-gray-800", base_classes),
        }
    };

    // Combined class computation for the status dot
    let status_dot_class = move || {
        let base_class = "w-2 h-2 rounded-full";
        match connection_status.get() {
            ConnectionStatus::Connected => format!("{} bg-green-500", base_class),
            ConnectionStatus::Connecting => format!("{} bg-yellow-500", base_class),
            ConnectionStatus::Error => format!("{} bg-red-500", base_class),
            ConnectionStatus::Disconnected => format!("{} bg-gray-500", base_class),
        }
    };

    view! {
        <div class="flex justify-center mb-4">
            <div class=status_div_class>
                <div class=status_dot_class></div>
                <span>{move || match connection_status.get() {
                    ConnectionStatus::Connected => "Connected",
                    ConnectionStatus::Connecting => "Connecting...",
                    ConnectionStatus::Error => "Connection Error",
                    ConnectionStatus::Disconnected => "Disconnected"
                }}</span>
            </div>
        </div>

        <Show when=move || error_message.get().is_some()>
            <div class="max-w-4xl mx-auto mb-6">
                <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
                    <strong>"Error: "</strong>
                    {move || error_message.get().unwrap_or_default()}
                </div>
            </div>
        </Show>
    }
}
