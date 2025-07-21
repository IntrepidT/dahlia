use crate::app::components::Header;
use crate::app::models::user::SessionUser;
use leptos::*;
use leptos_router::*;

#[component]
pub fn HomePage() -> impl IntoView {
    // Access the auth context signals directly
    let current_user =
        use_context::<ReadSignal<Option<SessionUser>>>().expect("Auth context not found");
    let loading = use_context::<ReadSignal<bool>>().expect("Auth context not found");

    view! {
        <Suspense fallback=move || view! { <p>"Loading..."</p> }>
            {move || {
                // If not loading and user is already logged in, redirect to dashboard
                if !loading.get() && current_user.get().is_some() {
                    view! { <Redirect path="/dashboard"/> }
                } else {
                    // Otherwise, show the enhanced homepage content
                    view! {
                        <div class="bg-[#F9F9F8] min-h-screen flex flex-col">
                            <Header />

                            // Hero Section
                            <main id="main-content" role="main" class="flex-1">
                                <section class="relative overflow-hidden">
                                    <div class="max-w-8xl mx-auto px-10">
                                        <div class="h-screen items-center justify-center rounded-2xl flex-col relative overflow-hidden">
                                            // Multi-layered background
                                            <div class="absolute inset-0 bg-gradient-to-br from-slate-700 via-gray-600 to-stone-600"></div>
                                            <div class="absolute inset-0 bg-[url('/assets/home23.png')] bg-cover bg-center"></div>

                                            <div class="h-5/6 pt-20 ml-20 mt-30 relative z-10">
                                                <h1 class="text-6xl font-extrabold text-left text-white mt-20 mb-10 drop-shadow-lg leading-tight">
                                                    Simplify Testing.<br/>Put Teachers First.
                                                </h1>
                                                <p class="text-2xl font-medium text-left text-white mt-10 drop-shadow-md leading-relaxed max-w-2xl">
                                                    A complete testing platform with real-time sessions,<br/>
                                                    automated grading, and comprehensive analytics.
                                                </p>
                                                <div class="flex gap-4 mt-12">
                                                    <A href="/login" class="font-semibold text-white">
                                                        <div class="bg-[#2E3A59] rounded-2xl border-white border-2 px-8 py-4 hover:bg-[#3a4660] transition-all duration-300 shadow-xl hover:shadow-2xl transform hover:-translate-y-1">
                                                            "Get Started"
                                                            <img src="/assets/arrow.png" alt="arrow" class="inline h-6 w-6 ml-2" />
                                                        </div>
                                                    </A>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </section>

                                // Core Features Section - Based on actual routes
                                <section class="py-24 bg-white">
                                    <div class="max-w-7xl mx-auto px-10">
                                        <div class="text-center mb-20">
                                            <h2 class="text-5xl font-bold text-gray-900 mb-6">
                                                "Complete testing solution"
                                            </h2>
                                            <p class="text-xl text-gray-600 max-w-3xl mx-auto leading-relaxed">
                                                "From test creation to real-time administration and detailed analytics - everything you need in one integrated platform."
                                            </p>
                                        </div>

                                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
                                            // Test Builder Feature
                                            <div class="bg-gradient-to-br from-blue-50 to-indigo-100 rounded-3xl p-8 h-full">
                                                <div class="w-16 h-16 bg-blue-600 rounded-2xl flex items-center justify-center mb-6">
                                                    <svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/>
                                                    </svg>
                                                </div>
                                                <h3 class="text-2xl font-bold text-gray-900 mb-4">"Advanced Test Builder"</h3>
                                                <p class="text-gray-600 leading-relaxed">
                                                    "Create sophisticated assessments with multiple question types, test variations, and custom templates including flashcard sets and grid tests."
                                                </p>
                                            </div>

                                            // Real-time Testing Feature
                                            <div class="bg-gradient-to-br from-green-50 to-emerald-100 rounded-3xl p-8 h-full">
                                                <div class="w-16 h-16 bg-green-600 rounded-2xl flex items-center justify-center mb-6">
                                                    <svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
                                                    </svg>
                                                </div>
                                                <h3 class="text-2xl font-bold text-gray-900 mb-4">"Live Test Sessions"</h3>
                                                <p class="text-gray-600 leading-relaxed">
                                                    "Administer tests in real-time with live monitoring, instant feedback, and support for both authenticated and anonymous student participation."
                                                </p>
                                            </div>

                                            // Gradebook & Analytics Feature
                                            <div class="bg-gradient-to-br from-purple-50 to-pink-100 rounded-3xl p-8 h-full">
                                                <div class="w-16 h-16 bg-purple-600 rounded-2xl flex items-center justify-center mb-6">
                                                    <svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012-2z"/>
                                                    </svg>
                                                </div>
                                                <h3 class="text-2xl font-bold text-gray-900 mb-4">"Comprehensive Gradebook"</h3>
                                                <p class="text-gray-600 leading-relaxed">
                                                    "Track student progress with detailed analytics, automated grading, and comprehensive reporting across all assessments and test sessions."
                                                </p>
                                            </div>
                                        </div>
                                    </div>
                                </section>

                                // Platform Workflow Section
                                <section class="py-24 bg-gray-50">
                                    <div class="max-w-7xl mx-auto px-10">
                                        <div class="text-center mb-16">
                                            <h2 class="text-4xl font-bold text-gray-900 mb-6">
                                                "How it works"
                                            </h2>
                                            <p class="text-xl text-gray-600 max-w-2xl mx-auto">
                                                "A streamlined workflow from test creation to results analysis"
                                            </p>
                                        </div>

                                        <div class="grid grid-cols-1 md:grid-cols-4 gap-8">
                                            // Step 1
                                            <div class="text-center">
                                                <div class="w-16 h-16 bg-blue-600 rounded-full flex items-center justify-center mx-auto mb-6 text-white font-bold text-xl">
                                                    "1"
                                                </div>
                                                <h3 class="text-xl font-semibold text-gray-900 mb-3">"Build Tests"</h3>
                                                <p class="text-gray-600 text-sm leading-relaxed">
                                                    "Use our test builder to create assessments with multiple question types and variations"
                                                </p>
                                            </div>

                                            // Step 2
                                            <div class="text-center">
                                                <div class="w-16 h-16 bg-green-600 rounded-full flex items-center justify-center mx-auto mb-6 text-white font-bold text-xl">
                                                    "2"
                                                </div>
                                                <h3 class="text-xl font-semibold text-gray-900 mb-3">"Launch Sessions"</h3>
                                                <p class="text-gray-600 text-sm leading-relaxed">
                                                    "Start real-time test sessions with live monitoring and student management"
                                                </p>
                                            </div>

                                            // Step 3
                                            <div class="text-center">
                                                <div class="w-16 h-16 bg-purple-600 rounded-full flex items-center justify-center mx-auto mb-6 text-white font-bold text-xl">
                                                    "3"
                                                </div>
                                                <h3 class="text-xl font-semibold text-gray-900 mb-3">"Monitor & Grade"</h3>
                                                <p class="text-gray-600 text-sm leading-relaxed">
                                                    "Track progress in real-time with automatic grading and immediate feedback"
                                                </p>
                                            </div>

                                            // Step 4
                                            <div class="text-center">
                                                <div class="w-16 h-16 bg-indigo-600 rounded-full flex items-center justify-center mx-auto mb-6 text-white font-bold text-xl">
                                                    "4"
                                                </div>
                                                <h3 class="text-xl font-semibold text-gray-900 mb-3">"Analyze Results"</h3>
                                                <p class="text-gray-600 text-sm leading-relaxed">
                                                    "Review detailed analytics and generate comprehensive reports"
                                                </p>
                                            </div>
                                        </div>
                                    </div>
                                </section>

                                // Test Format Showcase
                                <section class="py-24 bg-white">
                                    <div class="max-w-7xl mx-auto px-10">
                                        <div class="text-center mb-16">
                                            <h2 class="text-4xl font-bold text-gray-900 mb-6">
                                                "Multiple test formats supported"
                                            </h2>
                                            <p class="text-xl text-gray-600 max-w-3xl mx-auto">
                                                "Create engaging assessments that fit your teaching style and learning objectives"
                                            </p>
                                        </div>

                                        <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
                                            // Flashcard Sets
                                            <div class="bg-gradient-to-br from-orange-50 to-red-100 rounded-3xl p-8">
                                                <div class="w-16 h-16 bg-orange-600 rounded-2xl flex items-center justify-center mb-6">
                                                    <svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"/>
                                                    </svg>
                                                </div>
                                                <h3 class="text-2xl font-bold text-gray-900 mb-4">"Interactive Flashcard Sets"</h3>
                                                <p class="text-gray-600 leading-relaxed">
                                                    "Create digital flashcard assessments perfect for vocabulary, definitions, and quick recall exercises with immediate feedback."
                                                </p>
                                            </div>

                                            // Grid Tests
                                            <div class="bg-gradient-to-br from-teal-50 to-cyan-100 rounded-3xl p-8">
                                                <div class="w-16 h-16 bg-teal-600 rounded-2xl flex items-center justify-center mb-6">
                                                    <svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z"/>
                                                    </svg>
                                                </div>
                                                <h3 class="text-2xl font-bold text-gray-900 mb-4">"Advanced Grid Tests"</h3>
                                                <p class="text-gray-600 leading-relaxed">
                                                    "Design complex grid-based assessments ideal for mathematical problems, data analysis, and structured response formats."
                                                </p>
                                            </div>

                                            // Live Testing Sessions
                                            <div class="bg-gradient-to-br from-yellow-50 to-amber-100 rounded-3xl p-8">
                                                <div class="w-16 h-16 bg-yellow-600 rounded-2xl flex items-center justify-center mb-6">
                                                    <svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path d="M17 19a1 1 0 0 1-1-1v-2a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2v2a1 1 0 0 1-1 1z"/>
                                                        <path d="M17 21v-2"/>
                                                        <path d="M19 14V6.5a1 1 0 0 0-7 0v11a1 1 0 0 1-7 0V10"/>
                                                        <path d="M21 21v-2"/>
                                                        <path d="M3 5V3"/>
                                                        <path d="M4 10a2 2 0 0 1-2-2V6a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2a2 2 0 0 1-2 2z"/>
                                                        <path d="M7 5V3"/>
                                                    </svg>
                                                </div>
                                                <h3 class="text-2xl font-bold text-gray-900 mb-4">"Real-Time Live Testing"</h3>
                                                <p class="text-gray-600 leading-relaxed">
                                                    "Take your tests live with the click of a button: real-time testing across multiple your devices."
                                                </p>
                                            </div>
                                        </div>
                                    </div>
                                </section>

                                // Role-Based Access Section
                                <section class="py-24 bg-gray-50">
                                    <div class="max-w-7xl mx-auto px-10">
                                        <div class="text-center mb-16">
                                            <h2 class="text-4xl font-bold text-gray-900 mb-6">
                                                "Built for all educators"
                                            </h2>
                                            <p class="text-xl text-gray-600 max-w-2xl mx-auto">
                                                "Different interfaces and capabilities for teachers, administrators, and students"
                                            </p>
                                        </div>

                                        <div class="grid grid-cols-1 md:grid-cols-3 gap-8">
                                            // Teacher Dashboard
                                            <div class="bg-white rounded-3xl p-8 shadow-lg border border-gray-100">
                                                <div class="w-16 h-16 bg-blue-600 rounded-2xl flex items-center justify-center mx-auto mb-6">
                                                    <svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/>
                                                    </svg>
                                                </div>
                                                <h3 class="text-xl font-semibold text-gray-900 mb-4 text-center">"Teacher Dashboard"</h3>
                                                <ul class="text-gray-600 space-y-2 text-sm">
                                                    <li>"• Create and manage tests"</li>
                                                    <li>"• Launch live sessions"</li>
                                                    <li>"• Monitor student progress"</li>
                                                    <li>"• Access detailed gradebook"</li>
                                                    <li>"• Review test results"</li>
                                                </ul>
                                            </div>

                                            // Admin Panel
                                            <div class="bg-white rounded-3xl p-8 shadow-lg border border-gray-100">
                                                <div class="w-16 h-16 bg-purple-600 rounded-2xl flex items-center justify-center mx-auto mb-6">
                                                    <svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/>
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
                                                    </svg>
                                                </div>
                                                <h3 class="text-xl font-semibold text-gray-900 mb-4 text-center">"Admin Controls"</h3>
                                                <ul class="text-gray-600 space-y-2 text-sm">
                                                    <li>"• Manage teachers and students"</li>
                                                    <li>"• Configure SAML authentication"</li>
                                                    <li>"• System-wide test administration"</li>
                                                    <li>"• Advanced analytics dashboard"</li>
                                                    <li>"• Anonymized Student Data"</li>
                                                </ul>
                                            </div>

                                            // Student Experience
                                            <div class="bg-white rounded-3xl p-8 shadow-lg border border-gray-100">
                                                <div class="w-16 h-16 bg-green-600 rounded-2xl flex items-center justify-center mx-auto mb-6">
                                                    <svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 6.253v13m0-13C10.832 5.477 9.246 5 7.5 5S4.168 5.477 3 6.253v13C4.168 18.477 5.754 18 7.5 18s3.332.477 4.5 1.253m0-13C13.168 5.477 14.754 5 16.5 5c1.746 0 3.332.477 4.5 1.253v13C19.832 18.477 18.246 18 16.5 18c-1.746 0-3.332.477-4.5 1.253"/>
                                                    </svg>
                                                </div>
                                                <h3 class="text-xl font-semibold text-gray-900 mb-4 text-center">"Student Tracking"</h3>
                                                <ul class="text-gray-600 space-y-2 text-sm">
                                                    <li>"• Clean, distraction-free testing"</li>
                                                    <li>"• Anonymous or authenticated access"</li>
                                                    <li>"• Identify at risk students easily"</li>
                                                    <li>"• Design personalized progression plans"</li>
                                                    <li>"• Watch your students grow"</li>
                                                </ul>
                                            </div>
                                        </div>
                                    </div>
                                </section>

                                // CTA Section
                                <section class="py-24 bg-gradient-to-br from-slate-700 via-gray-700 to-stone-700">
                                    <div class="max-w-4xl mx-auto px-10 text-center">
                                        <h2 class="text-5xl font-bold text-white mb-6">
                                            "Ready to streamline your testing?"
                                        </h2>
                                        <p class="text-xl text-white/90 mb-12 leading-relaxed">
                                            "Join educators who are already using our comprehensive platform to create, administer, and analyze tests with unprecedented ease and insight."
                                        </p>

                                        <div class="flex flex-col sm:flex-row gap-6 justify-center">
                                            <A href="/login" class="font-semibold">
                                                <div class="bg-white text-slate-700 rounded-2xl px-10 py-4 hover:bg-gray-100 transition-all duration-300 shadow-xl hover:shadow-2xl transform hover:-translate-y-1">
                                                    "Sign In to Get Started"
                                                </div>
                                            </A>
                                        </div>
                                    </div>
                                </section>
                            </main>
                        </div>
                    }.into_view()
                }
            }}
        </Suspense>
    }
}
