mod components;
use std::borrow::Borrow;

#[yew::function_component(App)]
fn app() -> yew::Html {
    log::info!("rendering...");

    // state of the app
    let languages = yew::use_state_eq(|| vec![]);
    let selected_input_language = yew::use_mut_ref(move || None);
    let selected_output_language = yew::use_mut_ref(move || None);
    let input_text = yew::use_mut_ref(move || "".into());
    let output_text_area_node = yew::use_node_ref();

    // retrieve available languages once
    {
        let languages = languages.clone();
        yew::use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match gloo_net::http::Request::get("/languages").send().await {
                    Ok(response) if response.ok() => match response.json().await {
                        Ok(langs) => {
                            log::info!("retrieved languages: {:?}", langs);
                            languages.set(langs);
                        }
                        Err(e) => log::error!("could not parse languages: {}", e),
                    },
                    Ok(response) => log::error!(
                        "could not retrieve languages: {} ({})",
                        response.status(),
                        response.status_text()
                    ),
                    Err(e) => log::error!("could not retrieve languages: {}", e),
                }
            });
        });
    }

    // handle translation
    let on_translate_button_clicked = {
        let selected_input_language = selected_input_language.clone();
        let selected_output_language = selected_output_language.clone();
        let input_text = input_text.clone();
        let output_text_area_node = output_text_area_node.clone();
        yew::Callback::from(move |_: web_sys::MouseEvent| {
            let selected_input_language: &std::cell::RefCell<Option<common::Language>> =
                selected_input_language.borrow();
            let selected_output_language: &std::cell::RefCell<Option<common::Language>> =
                selected_output_language.borrow();
            let input_text: &std::cell::RefCell<String> = input_text.borrow();

            if let (Some(input_language), Some(output_language), input_text) = (
                selected_input_language.borrow().as_ref(),
                selected_output_language.borrow().as_ref(),
                <String as AsRef<str>>::as_ref(&input_text.borrow()),
            ) {
                let request = common::TranslationRequest {
                    source_language_code: input_language.code.clone(),
                    target_language_code: output_language.code.clone(),
                    text: input_text.into(),
                };

                // send translation request
                match gloo_net::http::Request::post("/translate").json(&request) {
                    Err(e) => {
                        log::error!("could not serialize translation request: {}", e);
                    }
                    Ok(request) => {
                        let output_text_area_node = output_text_area_node.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            match request.send().await {
                                Ok(response) if response.ok() => {
                                    // translation successful -> set output value
                                    if let (Some(output_text_area_node), Ok(translation)) = (
                                        output_text_area_node
                                            .cast::<web_sys::HtmlTextAreaElement>(),
                                        response.json::<String>().await,
                                    ) {
                                        output_text_area_node.set_value(&translation);
                                    }
                                }
                                Ok(response) => log::error!(
                                    "could not translate: {} ({})",
                                    response.status(),
                                    response.status_text()
                                ),
                                Err(e) => log::error!("could not translate: {}", e),
                            }
                        })
                    }
                }
            };
        })
    };

    // assemble final html structure
    yew::html! {
        <>
            <components::InputField {input_text} placeholder_text="input text" />

            <components::LanguageSelect languages={(*languages).clone()} selected_language={selected_input_language} />
            <button onclick={on_translate_button_clicked}>{ "translate" }</button>
            <components::LanguageSelect languages={(*languages).clone()} selected_language={selected_output_language} />

            <textarea ref={output_text_area_node}></textarea>
        </>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
