use dioxus::prelude::*;

#[component]
pub fn Directory(segments: Vec<String>) -> Element {
    rsx! {
        div {
            "Directory page"
            ul {
                {segments.iter().map(|segment| rsx!(li { "{segment}" }))}
            }
        }
    }
}