use super::types::ConnectionStatus;
use leptos::*;

#[component]
pub fn ConnectionStatusIndicator(
    #[prop(into)] connection_status: Signal<ConnectionStatus>,
    #[prop(into)] error_message: Signal<Option<String>>,
) -> impl IntoView {
    view! {
        <div class="flex justify-center mb-4">
            <div class="flex items-center space-x-2 px-3 py-1 rounded-full text-sm"
                 class:bg-green-100={move || matches!(connection_status.get(), ConnectionStatus::Connected)}
                 class:text-green-800={move || matches!(connection_status.get(), ConnectionStatus::Connected)}
                 class:bg-yellow-100={move || matches!(connection_status.get(), ConnectionStatus::Connecting)}
                 class:text-yellow-800={move || matches!(connection_status.get(), ConnectionStatus::Connecting)}
                 class:bg-red-100={move || matches!(connection_status.get(), ConnectionStatus::Error)}
                 class:text-red-800={move || matches!(connection_status.get(), ConnectionStatus::Error)}
                 class:bg-gray-100={move || matches!(connection_status.get(), ConnectionStatus::Disconnected)}
                 class:text-gray-800={move || matches!(connection_status.get(), ConnectionStatus::Disconnected)}>
                <div class="w-2 h-2 rounded-full"
                     class:bg-green-500={move || matches!(connection_status.get(), ConnectionStatus::Connected)}
                     class:bg-yellow-500={move || matches!(connection_status.get(), ConnectionStatus::Connecting)}
                     class:bg-red-500={move || matches!(connection_status.get(), ConnectionStatus::Error)}
                     class:bg-gray-500={move || matches!(connection_status.get(), ConnectionStatus::Disconnected)}></div>
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
