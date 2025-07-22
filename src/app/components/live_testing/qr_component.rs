// QR Code component for easy mobile access
use leptos::*;
use uuid::Uuid;

#[component]
pub fn TestShareModal(
    #[prop(into)] show: Signal<bool>,
    #[prop(into)] test_id: Signal<String>,
    #[prop(into)] room_id: Signal<Option<Uuid>>,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    let (copied_link, set_copied_link) = create_signal(false);
    let (show_qr, set_show_qr) = create_signal(false);

    let student_link = create_memo(move |_| {
        if let (Some(room), tid) = (room_id.get(), test_id.get()) {
            if !tid.is_empty() {
                let origin = web_sys::window()
                    .and_then(|w| w.location().origin().ok())
                    .unwrap_or_else(|| "http://localhost:8080".to_string());
                return format!("{}/student-test/{}/{}", origin, tid, room);
            }
        }
        String::new()
    });

    // Generate QR code URL (using qr-server.com API)
    let qr_code_url = create_memo(move |_| {
        let link = student_link.get();
        if !link.is_empty() {
            format!(
                "https://api.qrserver.com/v1/create-qr-code/?size=200x200&data={}",
                urlencoding::encode(&link)
            )
        } else {
            String::new()
        }
    });

    let copy_link = move |_| {
        let link = student_link.get();
        if !link.is_empty() {
            #[cfg(feature = "hydrate")]
            {
                if let Some(navigator) = web_sys::window().and_then(|w| w.navigator().clipboard()) {
                    let _ = navigator.write_text(&link);
                    set_copied_link.set(true);

                    set_timeout_with_handle(
                        move || set_copied_link.set(false),
                        std::time::Duration::from_secs(2),
                    )
                    .ok();
                }
            }
        }
    };

    view! {
        <Show when=move || show.get()>
            <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
                 on:click=move |_| on_close.call(())>
                <div class="bg-white rounded-lg p-6 max-w-lg w-full mx-4"
                     on:click=move |e| e.stop_propagation()>

                    <div class="flex justify-between items-center mb-4">
                        <h3 class="text-lg font-medium">"Share Test with Student"</h3>
                        <button
                            class="text-gray-400 hover:text-gray-600"
                            on:click=move |_| on_close.call(())
                        >
                            "✕"
                        </button>
                    </div>

                    <div class="space-y-4">
                        // Tab navigation
                        <div class="flex border-b">
                            <button
                                class="px-4 py-2 font-medium"
                                class:border-b-2={move || !show_qr.get()}
                                class:border-blue-500={move || !show_qr.get()}
                                class:text-blue-600={move || !show_qr.get()}
                                on:click=move |_| set_show_qr.set(false)
                            >
                                "Share Link"
                            </button>
                            <button
                                class="px-4 py-2 font-medium"
                                class:border-b-2={move || show_qr.get()}
                                class:border-blue-500={move || show_qr.get()}
                                class:text-blue-600={move || show_qr.get()}
                                on:click=move |_| set_show_qr.set(true)
                            >
                                "QR Code"
                            </button>
                        </div>

                        // Link sharing tab
                        <Show when=move || !show_qr.get()>
                            <div class="space-y-4">
                                <p class="text-sm text-gray-600">
                                    "Send this link to your student. They can join directly without creating an account:"
                                </p>

                                <div class="bg-gray-50 p-3 rounded border break-all text-sm">
                                    {move || student_link.get()}
                                </div>

                                <div class="flex gap-2">
                                    <button
                                        class="flex-1 px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors"
                                        on:click=copy_link
                                    >
                                        {move || if copied_link.get() { "✓ Copied!" } else { "📋 Copy Link" }}
                                    </button>

                                    // Email sharing
                                    <a
                                        href=move || format!(
                                            "mailto:?subject=Join%20Test%20Session&body=Please%20join%20my%20test%20session%20by%20clicking%20this%20link:%20{}",
                                            urlencoding::encode(&student_link.get())
                                        )
                                        class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-700 transition-colors"
                                    >
                                        "📧 Email"
                                    </a>
                                </div>
                            </div>
                        </Show>

                        // QR Code tab
                        <Show when=move || show_qr.get()>
                            <div class="space-y-4 text-center">
                                <p class="text-sm text-gray-600">
                                    "Have your student scan this QR code with their phone:"
                                </p>

                                <div class="flex justify-center">
                                    <img
                                        src=move || qr_code_url.get()
                                        alt="QR Code for test session"
                                        class="border rounded-lg shadow-sm"
                                        style="max-width: 200px; max-height: 200px;"
                                    />
                                </div>

                                <div class="text-xs text-gray-500">
                                    <p>"📱 Works with any QR code scanner or camera app"</p>
                                </div>
                            </div>
                        </Show>

                        // Instructions
                        <div class="bg-blue-50 p-4 rounded-lg">
                            <h4 class="font-medium text-blue-900 mb-2">"How it works:"</h4>
                            <ol class="text-sm text-blue-800 space-y-1">
                                <li>"1. Share the link or QR code with your student"</li>
                                <li>"2. Student enters their name and ID"</li>
                                <li>"3. Student automatically joins your test session"</li>
                                <li>"4. Start the test when ready!"</li>
                            </ol>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}
