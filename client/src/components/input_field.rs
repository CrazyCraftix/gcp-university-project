use yew::TargetCast as _;

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
pub struct Language {
    pub id: usize,
    pub display_string: String,
}

#[derive(yew::Properties, PartialEq)]
pub struct Props {
    pub placeholder_text: String,
    pub input_text: std::rc::Rc<std::cell::RefCell<String>>,
}

#[yew::function_component(InputField)]
pub fn input_field(props: &Props) -> yew::Html {
    // callback to update the input text on changes
    let text_changed = {
        let input_text = props.input_text.clone();
        yew::Callback::from(move |event: yew::Event| {
            if let Some(text_element) = event.target_dyn_into::<web_sys::HtmlTextAreaElement>() {
                input_text.replace(text_element.value());
            } else {
                input_text.replace("".into());
            }
        })
    };

    // construct
    yew::html! {
        <textarea onchange={text_changed} placeholder={props.placeholder_text.clone()} >
            { &input_text }
        </textarea>
    }
}
