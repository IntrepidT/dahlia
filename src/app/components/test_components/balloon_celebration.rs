use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use wasm_bindgen::prelude::*;

#[component]
pub fn BalloonCelebration(
    #[prop(into)] show: Signal<bool>,
    #[prop(default = "🎉 Test Complete! Great job! 🎉")] message: &'static str,
    #[prop(default = 5000)] duration: u32, // Duration in milliseconds
) -> impl IntoView {
    let (is_visible, set_is_visible) = signal(false);
    let (animate_out, set_animate_out) = signal(false);

    // Handle show/hide animation
    Effect::new(move |_| {
        if show.get() {
            set_is_visible.set(true);
            set_animate_out.set(false);

            // Auto-hide after duration
            #[cfg(feature = "hydrate")]
            {
                use wasm_bindgen::closure::Closure;

                let timeout = web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        &Closure::once_into_js(move || {
                            set_animate_out.set(true);
                            // Give time for exit animation
                            let hide_timeout = web_sys::window()
                                .unwrap()
                                .set_timeout_with_callback_and_timeout_and_arguments_0(
                                    &Closure::once_into_js(move || {
                                        set_is_visible.set(false);
                                    })
                                    .into(),
                                    500, // Exit animation duration
                                )
                                .unwrap();
                        })
                        .into(),
                        duration as i32,
                    )
                    .unwrap();
            }

            #[cfg(not(feature = "hydrate"))]
            {
                // For server-side rendering, just set a simple timeout using leptos
                set_timeout(
                    move || {
                        set_animate_out.set(true);
                        set_timeout(
                            move || {
                                set_is_visible.set(false);
                            },
                            std::time::Duration::from_millis(500),
                        );
                    },
                    std::time::Duration::from_millis(duration as u64),
                );
            }
        } else {
            set_is_visible.set(false);
            set_animate_out.set(false);
        }
    });

    view! {
        <Show when=move || is_visible.get()>
            <div class=move || {
                let base_classes = "fixed inset-0 z-50 flex items-center justify-center pointer-events-none";
                if animate_out.get() {
                    format!("{} animate-fade-out", base_classes)
                } else {
                    format!("{} animate-fade-in", base_classes)
                }
            }>
                {/* Background overlay */}
                <div class="absolute inset-0 bg-black bg-opacity-20 backdrop-blur-sm"></div>

                {/* Celebration content */}
                <div class="relative z-10 text-center px-8">
                    {/* Main message */}
                    <div class=move || {
                        if animate_out.get() {
                            "transform transition-all duration-500 ease-in-out scale-75 opacity-0"
                        } else {
                            "transform transition-all duration-700 ease-out scale-100 opacity-100 animate-bounce-gentle"
                        }
                    }>
                        <div class="bg-white rounded-2xl shadow-2xl p-8 mb-8 border-4 border-yellow-300">
                            <h1 class="text-4xl md:text-6xl font-bold text-transparent bg-clip-text bg-gradient-to-r from-purple-600 to-pink-600 mb-4">
                                {message}
                            </h1>
                            <p class="text-xl text-gray-600 font-medium">
                                "You can now close this page or wait for your teacher's next instructions."
                            </p>
                        </div>
                    </div>

                    {/* Floating balloons */}
                    <div class="absolute inset-0 pointer-events-none">
                        <Balloon color="bg-red-400" delay="0s" x_position="10%" />
                        <Balloon color="bg-blue-400" delay="0.5s" x_position="20%" />
                        <Balloon color="bg-yellow-400" delay="1s" x_position="30%" />
                        <Balloon color="bg-green-400" delay="0.3s" x_position="70%" />
                        <Balloon color="bg-purple-400" delay="0.8s" x_position="80%" />
                        <Balloon color="bg-pink-400" delay="1.2s" x_position="90%" />
                        <Balloon color="bg-indigo-400" delay="0.2s" x_position="50%" />
                        <Balloon color="bg-orange-400" delay="1.5s" x_position="60%" />
                    </div>

                    {/* Confetti particles */}
                    <div class="absolute inset-0 pointer-events-none overflow-hidden">
                        <ConfettiParticle color="bg-red-500" delay="0s" x="15%" />
                        <ConfettiParticle color="bg-blue-500" delay="0.3s" x="25%" />
                        <ConfettiParticle color="bg-yellow-500" delay="0.6s" x="35%" />
                        <ConfettiParticle color="bg-green-500" delay="0.9s" x="45%" />
                        <ConfettiParticle color="bg-purple-500" delay="1.2s" x="55%" />
                        <ConfettiParticle color="bg-pink-500" delay="1.5s" x="65%" />
                        <ConfettiParticle color="bg-indigo-500" delay="0.4s" x="75%" />
                        <ConfettiParticle color="bg-orange-500" delay="0.7s" x="85%" />
                    </div>
                </div>
            </div>
        </Show>

        {/* Custom CSS animations */}
        <style>
            r#"
            @keyframes float-up {
                0% {
                    transform: translateY(100vh) scale(0);
                    opacity: 0;
                }
                10% {
                    opacity: 1;
                }
                90% {
                    opacity: 1;
                }
                100% {
                    transform: translateY(-100vh) scale(1);
                    opacity: 0;
                }
            }
            
            @keyframes confetti-fall {
                0% {
                    transform: translateY(-100vh) rotate(0deg);
                    opacity: 1;
                }
                100% {
                    transform: translateY(100vh) rotate(360deg);
                    opacity: 0;
                }
            }
            
            @keyframes bounce-gentle {
                0%, 20%, 50%, 80%, 100% {
                    transform: translateY(0);
                }
                40% {
                    transform: translateY(-10px);
                }
                60% {
                    transform: translateY(-5px);
                }
            }
            
            @keyframes fade-in {
                from {
                    opacity: 0;
                    transform: scale(0.8);
                }
                to {
                    opacity: 1;
                    transform: scale(1);
                }
            }
            
            @keyframes fade-out {
                from {
                    opacity: 1;
                    transform: scale(1);
                }
                to {
                    opacity: 0;
                    transform: scale(0.8);
                }
            }
            
            .animate-float-up {
                animation: float-up 4s ease-out forwards;
            }
            
            .animate-confetti-fall {
                animation: confetti-fall 3s linear forwards;
            }
            
            .animate-bounce-gentle {
                animation: bounce-gentle 2s ease-in-out infinite;
            }
            
            .animate-fade-in {
                animation: fade-in 0.5s ease-out forwards;
            }
            
            .animate-fade-out {
                animation: fade-out 0.5s ease-in forwards;
            }
            "#
        </style>
    }
}

#[component]
fn Balloon(
    #[prop(into)] color: &'static str,
    #[prop(into)] delay: &'static str,
    #[prop(into)] x_position: &'static str,
) -> impl IntoView {
    view! {
        <div
            class=format!("absolute bottom-0 w-12 h-16 {} rounded-full animate-float-up", color)
            style=format!("left: {}; animation-delay: {}", x_position, delay)
        >
            {/* Balloon string */}
            <div class="absolute top-full left-1/2 w-0.5 h-20 bg-gray-400 transform -translate-x-1/2"></div>

            {/* Balloon highlight */}
            <div class="absolute top-2 left-2 w-3 h-4 bg-white bg-opacity-40 rounded-full"></div>
        </div>
    }
}

#[component]
fn ConfettiParticle(
    #[prop(into)] color: &'static str,
    #[prop(into)] delay: &'static str,
    #[prop(into)] x: &'static str,
) -> impl IntoView {
    view! {
        <div
            class=format!("absolute top-0 w-2 h-2 {} animate-confetti-fall", color)
            style=format!("left: {}; animation-delay: {}", x, delay)
        ></div>
    }
}

// Quick celebration trigger for instant feedback
#[component]
pub fn QuickCelebration(#[prop(into)] show: Signal<bool>) -> impl IntoView {
    view! {
        <Show when=move || show.get()>
            <div class="fixed top-4 right-4 z-50 animate-bounce">
                <div class="bg-green-500 text-white px-6 py-3 rounded-lg shadow-lg flex items-center gap-2">
                    <span class="text-2xl">"🎉"</span>
                    <span class="font-semibold">"Test Submitted!"</span>
                </div>
            </div>
        </Show>
    }
}
