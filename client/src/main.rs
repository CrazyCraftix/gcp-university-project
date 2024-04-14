mod components;
use std::borrow::Borrow;

use components::language_select::Language;

#[yew::function_component(App)]
fn app() -> yew::Html {
    let languages = vec![
        Language {
            id: 0,
            display_string: "English".into(),
        },
        Language {
            id: 1,
            display_string: "French".into(),
        },
        Language {
            id: 2,
            display_string: "Spanish".into(),
        },
        Language {
            id: 3,
            display_string: "German".into(),
        },
    ];

    // state of the app
    let selected_input_language = yew::use_mut_ref(move || None);
    let selected_output_language = yew::use_mut_ref(move || None);
    let input_text = yew::use_mut_ref(move || "".into());
    let output_text_area_node = yew::use_node_ref();

    let on_translate_button_clicked = {
        let selected_input_language = selected_input_language.clone();
        let selected_output_language = selected_output_language.clone();
        let input_text = input_text.clone();
        let output_text_area_node = output_text_area_node.clone();
        yew::Callback::from(move |_: web_sys::MouseEvent| {
            let selected_input_language: &std::cell::RefCell<Option<Language>> =
                selected_input_language.borrow();
            let selected_output_language: &std::cell::RefCell<Option<Language>> =
                selected_output_language.borrow();
            let input_text: &std::cell::RefCell<String> = input_text.borrow();

            let output_text = if let (Some(input_language), Some(output_language), input_text) = (
                selected_input_language.borrow().as_ref(),
                selected_output_language.borrow().as_ref(),
                <String as AsRef<str>>::as_ref(&input_text.borrow()),
            ) {
                let output = format!(
                    "translate was clicked!\n- input language: {:?}\n- output language: {:?}\n- input text: {}",
                    input_language, output_language, input_text
                );
                log::info!("{output}");
                output
            } else {
                "".into()
            };

            if let Some(output_text_area_node) =
                output_text_area_node.cast::<web_sys::HtmlTextAreaElement>()
            {
                output_text_area_node.set_value(&output_text);
            }
        })
    };

    log::info!("rendering...");

    yew::html! {
        <>
            <components::InputField {input_text} placeholder_text="input text" />
            <components::LanguageSelect languages={languages.clone()} selected_language={selected_input_language} />
            <button onclick={on_translate_button_clicked}>{ "translate" }</button>
            <components::LanguageSelect {languages} selected_language={selected_output_language} />
            <textarea ref={output_text_area_node}></textarea>
        </>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
