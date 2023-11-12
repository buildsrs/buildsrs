//! # Buildsrs Frontend
//!
//! This crate implements the frontend of buildsrs. It is implemented using the
//! [Yew](https://yew.rs) framework, which uses a component model similar to the well-known
//! React framework.
//!
//! It communicates with the backend using the REST API that the backend provides.

use yew::prelude::*;

#[function_component]
pub fn App() -> Html {
    html! {
        <div class="flex flex-col min-h-screen">
            <header class="bg-green-800 text-yellow-200">
                <div class="max-w-4xl mx-auto p-4 mb-4">
                    <nav class="flex items-center gap-2">
                        <img src="/static/builder.svg" class="w-8 h-8" />
                        <span class="text-3xl font-bold">{"builds.rs"}</span>
                        <span class="grow"></span>
                        <a href="#builds">{"Browse all builds"}</a>
                        <span class="text-yellow-500">{"|"}</span>
                        <a href="https://github.com/buildsrs/buildsrs">{"Source"}</a>
                    </nav>
                    <div class="text-center text-4xl font-bold my-6">
                        {"Build service for crates on crates.io"}
                    </div>
                    <div class="text-center text-black my-6">
                        <input type="text" class="rounded-xl w-[70%] p-2 px-4" />
                    </div>
                </div>
            </header>
            <main class="bg-amber-200 grow">
                <div class="max-w-4xl mx-auto p-4">
                    {"Work-in-progress! Check out the GitHub project to contribute."}
                </div>
            </main>
            <footer class="bg-green-800 text-yellow-200">
                <div class="max-w-4xl mx-auto p-4 grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4">
                    <div>{"Rust"}</div>
                    <div>{"Get help"}</div>
                    <div>{"Policies"}</div>
                    <div>{"Social"}</div>
                </div>
            </footer>
        </div>
    }
}
