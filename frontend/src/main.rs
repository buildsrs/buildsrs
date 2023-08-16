use yew::prelude::*;

#[function_component]
fn App() -> Html {
    html! {
        <div class="flex flex-col min-h-screen">
            <header class="bg-yellow-600">
                <div class="max-w-4xl mx-auto p-4">
                    <nav class="flex items-center gap-2">
                        <img src="/static/builder.svg" class="w-8 h-8" />
                        <span class="text-3xl font-bold">{"builds.rs"}</span>
                        <span class="grow"></span>
                        <span>{"Browse all crates"}</span>
                    </nav>
                    <div class="text-center text-4xl font-bold my-6">
                        {"Build service for Rust crates on crates.io"}
                    </div>
                    <div class="text-center my-6">
                        <input type="text" class="rounded-xl w-[70%] p-2 px-4" />
                    </div>
                </div>
            </header>
            <main class="bg-amber-200 grow">
                <div class="max-w-4xl mx-auto p-4">
                    {"Content"}
                </div>
            </main>
            <footer class="bg-yellow-600">
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

fn main() {
    yew::Renderer::<App>::new().render();
}
