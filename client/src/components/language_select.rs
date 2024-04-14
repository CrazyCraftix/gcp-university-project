use yew::TargetCast as _;

#[derive(yew::Properties, PartialEq)]
pub struct Props {
    pub languages: Vec<common::Language>,
    pub selected_language: std::rc::Rc<std::cell::RefCell<Option<common::Language>>>,
}

#[yew::function_component(LanguageSelect)]
pub fn language_select(props: &Props) -> yew::Html {
    // select first language, if nothing is selected
    if props.selected_language.borrow().is_none() {
        if let Some(first_language) = props.languages.first() {
            props
                .selected_language
                .replace(Some(first_language.clone()));
        }
    }

    // callback to update selected language if it changes
    let selection_changed = {
        let languages = props.languages.clone();
        let selected_language = props.selected_language.clone();
        yew::Callback::from(move |event: yew::Event| {
            if let Some(select_element) = event.target_dyn_into::<web_sys::HtmlSelectElement>() {
                let index = select_element.selected_index();
                if let Some(language) = languages.get(index as usize) {
                    selected_language.replace(Some(language.clone()));
                    return;
                }
            }
            selected_language.replace(None);
        })
    };

    // construct html elements for all the languages
    let language_elements = props
        .languages
        .iter()
        .map(|language| {
            let selected = props
                .selected_language
                .borrow()
                .as_ref()
                .map_or(false, |selected_language| selected_language == language);
            yew::html! {
                <option key={language.id} {selected}>{ language.display_string.clone() }</option>
            }
        })
        .collect::<yew::Html>();

    // construct
    yew::html! {
        <select onchange={selection_changed} >
            { language_elements }
        </select>
    }
}
