use super::types::{ConnectionStatus, Role};
use leptos::attr::value;
use leptos::prelude::*;
use uuid::Uuid;

#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::*;

#[component]
pub fn TestControls(
    #[prop(into)] role: Signal<Role>,
    #[prop(into)] is_test_active: Signal<bool>,
    #[prop(into)] is_submitted: Signal<bool>,
    #[prop(into)] connection_status: Signal<ConnectionStatus>,
    #[prop(into)] selected_student_id: Signal<Option<i32>>,
    #[prop(into)] room_id: Signal<Option<Uuid>>,
    #[prop(into)] test_id: Signal<String>,
    #[prop(into)] start_test: Callback<()>,
    #[prop(into)] end_test: Callback<()>,
) -> impl IntoView {
    let (show_link_modal, set_show_link_modal) = signal(false);
    let (copied_link, set_copied_link) = signal(false);
    let (show_qr, set_show_qr) = signal(false);

    // Generate student link
    let student_link = Memo::new(move |_| {
        if let (Some(room), tid) = (room_id.get(), test_id.get()) {
            if !tid.is_empty() {
                #[cfg(feature = "hydrate")]
                {
                    let origin = web_sys::window()
                        .and_then(|w| w.location().origin().ok())
                        .unwrap_or_else(|| "http://localhost:3000".to_string());
                    return format!("{}/student-test/{}/{}", origin, tid, room);
                }
                #[cfg(not(feature = "hydrate"))]
                {
                    return format!("http://localhost:3000/student-test/{}/{}", tid, room);
                }
            }
        }
        String::new()
    });

    // Generate QR code URL (using qr-server.com API)
    let qr_code_url = Memo::new(move |_| {
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

    // Copy link to clipboard - FIXED navigator.clipboard() call
    let copy_link = move |_| {
        let link = student_link.get();
        if !link.is_empty() {
            #[cfg(feature = "hydrate")]
            {
                // Fixed: navigator().clipboard() returns Clipboard directly, not Option<Clipboard>
                if let Some(window) = web_sys::window() {
                    let clipboard = window.navigator().clipboard();
                    let _ = clipboard.write_text(&link);
                    set_copied_link.set(true);

                    // Reset copied state after 2 seconds
                    set_timeout_with_handle(
                        move || set_copied_link.set(false),
                        std::time::Duration::from_secs(2),
                    )
                    .ok();
                }
            }
        }
    };

    // Quick start for anonymous mode - FIXED: removed the argument expectation
    /*let quick_start_test = move |_| {
        // Start test without requiring student selection for anonymous mode
        start_test.run(());
    };*/

    view! {
        <Show when=move || matches!(role.get(), Role::Teacher)>
            <div class="mb-8 flex flex-wrap gap-4 justify-center">
                <Show when=move || !is_test_active.get() && !is_submitted.get()>
                    <div class="w-full mb-4 flex flex-col items-center space-y-4">
                        // Action buttons
                        <div class="flex flex-wrap gap-2 justify-center">
                            // Share Link Button
                            <button
                                class="px-5 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                                on:click=move |_| set_show_link_modal.set(true)
                                disabled=move || !matches!(connection_status.get(), ConnectionStatus::Connected) || student_link.get().is_empty()
                            >
                                <span>"🔗"</span>
                                "Share Test Link"
                            </button>

                            // Quick Start (Anonymous Mode) - FIXED: call with empty argument
                            /*<button
                                class="px-5 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                                on:click=move |ev| quick_start_test(ev)
                                disabled=move || !matches!(connection_status.get(), ConnectionStatus::Connected)
                            >
                                <span>"⚡"</span>
                                "Quick Start"
                            </button>*/

                            // Traditional Start
                            <button
                                class="px-5 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                                on:click=move |_| start_test.run(())
                                disabled=move || selected_student_id.get().is_none() || !matches!(connection_status.get(), ConnectionStatus::Connected)
                            >
                                <span>"🎯"</span>
                                "Start with Selected Student"
                            </button>
                        </div>

                        // Helper text
                        /*<div class="text-sm text-gray-600 text-center max-w-2xl">
                            <p class="mb-2">
                                <strong>"Share Test Link:"</strong> " Students join instantly without logging in"
                            </p>
                            <p class="mb-2">
                                <strong>"Quick Start:"</strong> " Begin immediately, students can join anytime"
                            </p>
                            <p>
                                <strong>"Traditional:"</strong> " Select a specific student first"
                            </p>
                        </div>*/
                    </div>
                </Show>

                <Show when=move || is_test_active.get() && !is_submitted.get()>
                    <div class="flex flex-col items-center space-y-4">
                        <button
                            class="px-5 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors flex items-center gap-2"
                            on:click=move |_| end_test.run(())
                        >
                            <span>"🛑"</span>
                            "End Test Session"
                        </button>

                        // Show link while test is active
                        <button
                            class="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors text-sm flex items-center gap-2"
                            on:click=move |_| set_show_link_modal.set(true)
                        >
                            <span>"🔗"</span>
                            "Share Student Link"
                        </button>
                    </div>
                </Show>
            </div>

            // Student Link Sharing Modal
            <Show when=move || show_link_modal.get()>
                <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50"
                     on:click=move |_| set_show_link_modal.set(false)>
                    <div class="bg-white rounded-lg p-6 max-w-lg w-full mx-4"
                         on:click=move |e| e.stop_propagation()>

                        <div class="flex justify-between items-center mb-4">
                            <h3 class="text-lg font-medium">"Share Test with Students"</h3>
                            <button
                                class="text-gray-400 hover:text-gray-600 text-xl"
                                on:click=move |_| set_show_link_modal.set(false)
                            >
                                "✕"
                            </button>
                        </div>

                        <div class="space-y-4">
                            // Tab navigation
                            <div class="flex border-b">
                                <button
                                    class=("px-4 py-2 font-medium transition-colors")
                                    class=(["border-blue-500", "border-b-2", "text-blue-600"], move || !show_qr.get())
                                    class=("text-gray-500", move || show_qr.get())
                                    on:click=move |_| set_show_qr.set(false)
                                >
                                    "📋 Share Link"
                                </button>
                                <button
                                    class=("px-4 py-2 font-medium transition-colors")
                                    class=(["border-b-2", "border-blue-500", "text-blue-600"], move || show_qr.get())
                                    class=("text-gray-500", move || !show_qr.get())
                                    on:click=move |_| set_show_qr.set(true)
                                >
                                    "📱 QR Code"
                                </button>
                            </div>

                            // Link sharing tab
                            <Show when=move || !show_qr.get()>
                                <div class="space-y-4">
                                    <p class="text-sm text-gray-600">
                                        "Students can join instantly by visiting this link. No login required - they just need to enter their name and student ID:"
                                    </p>

                                    <div class="bg-gray-50 p-3 rounded border break-all text-sm font-mono">
                                        {move || student_link.get()}
                                    </div>

                                    <div class="flex gap-2">
                                        <button
                                            class="flex-1 px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors flex items-center justify-center gap-2"
                                            on:click=copy_link
                                        >
                                            {move || if copied_link.get() {
                                                view! { <><span>"✓"</span> "Copied!"</> }.into_any()
                                            } else {
                                                view! { <><span>"📋"</span> "Copy Link"</> }.into_any()
                                            }}
                                        </button>

                                        // Email sharing
                                        <a
                                            href=move || format!(
                                                "mailto:?subject=Join%20Test%20Session&body=Please%20join%20my%20test%20session%20by%20clicking%20this%20link:%0A%0A{}%0A%0ANo%20login%20required%20-%20just%20enter%20your%20name%20and%20student%20ID%20when%20prompted.",
                                                urlencoding::encode(&student_link.get())
                                            )
                                            class="px-4 py-2 bg-gray-600 text-white rounded hover:bg-gray-700 transition-colors flex items-center gap-2"
                                        >
                                            <span>"📧"</span>
                                            "Email"
                                        </a>
                                    </div>
                                </div>
                            </Show>

                            // QR Code tab
                            <Show when=move || show_qr.get()>
                                <div class="space-y-4 text-center">
                                    <p class="text-sm text-gray-600">
                                        "Students can scan this QR code with their phone camera or any QR code app:"
                                    </p>

                                    <div class="flex justify-center">
                                        <img
                                            src=move || qr_code_url.get()
                                            alt="QR Code for test session"
                                            class="border rounded-lg shadow-sm bg-white p-2"
                                            style="max-width: 200px; max-height: 200px;"
                                        />
                                    </div>

                                    <div class="text-xs text-gray-500 space-y-1">
                                        <p>"📱 Works with iPhone Camera, Android Camera, or any QR scanner"</p>
                                        <p>"💡 Students will be taken directly to the test join page"</p>
                                    </div>
                                </div>
                            </Show>

                            // Instructions
                            <div class="bg-blue-50 p-4 rounded-lg">
                                <h4 class="font-medium text-blue-900 mb-2">"📝 How it works:"</h4>
                                <ol class="text-sm text-blue-800 space-y-1 list-decimal list-inside">
                                    <li>"Share the link or QR code with your students"</li>
                                    <li>"Students enter their name and student ID (no login needed)"</li>
                                    <li>"They automatically join your test session"</li>
                                    <li>"Start the test when all students have joined!"</li>
                                </ol>
                            </div>
                        </div>
                    </div>
                </div>
            </Show>
        </Show>
    }
}

// Utility function for timeouts
#[cfg(feature = "hydrate")]
fn set_timeout_with_handle<F>(f: F, delay: std::time::Duration) -> Result<TimeoutHandle, JsValue>
where
    F: FnOnce() + 'static,
{
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    let callback = Closure::once(Box::new(f) as Box<dyn FnOnce()>);
    let handle = web_sys::window()
        .unwrap()
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            delay.as_millis() as i32,
        )?;

    callback.forget();
    Ok(TimeoutHandle(handle))
}

#[cfg(feature = "hydrate")]
struct TimeoutHandle(i32);

#[cfg(feature = "hydrate")]
impl TimeoutHandle {
    fn clear(&self) {
        web_sys::window().unwrap().clear_timeout_with_handle(self.0);
    }
}

#[cfg(not(feature = "hydrate"))]
fn set_timeout_with_handle<F>(_f: F, _delay: std::time::Duration) -> Result<(), ()>
where
    F: FnOnce() + 'static,
{
    Ok(())
}
