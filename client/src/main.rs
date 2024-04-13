use gloo_net::http::Request;
use yew::prelude::*;

#[derive(Clone, PartialEq, serde::Deserialize)]
struct Video {
    id: usize,
    title: String,
    speaker: String,
    url: String,
}

#[derive(Properties, PartialEq)]
struct VideoListProps {
    videos: Vec<Video>,
    on_click: Callback<Video>,
}
#[function_component(VideoList)]
fn video_list(VideoListProps { videos, on_click }: &VideoListProps) -> Html {
    videos
        .iter()
        .map(|video| {
            let on_video_select = {
                let video = video.clone();
                let on_click = on_click.clone();
                Callback::from(move |_| on_click.emit(video.clone()))
            };

            html! {
                <p key={video.id} onclick={on_video_select}>{format!("{}: {}", video.speaker, video.title)}</p>
            }
        })
        .collect()
}

#[derive(Properties, PartialEq)]
struct VideoDetailsProps {
    video: Video,
}
#[function_component(VideoDetails)]
fn video_details(VideoDetailsProps { video }: &VideoDetailsProps) -> Html {
    html! {
        <div>
            <h3>{ video.title.clone() }</h3>
            <img src="https://via.placeholder.com/640x360.png?text=Video+Player+Placeholder" alt="video thumbnail" />
        </div>
    }
}

#[function_component(App)]
fn app() -> Html {
    let videos = use_state_eq(|| vec![]);
    {
        let videos = videos.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                //let fetched_videos: Vec<Video> = Request::get("https://yew.rs/tutorial/data.json")
                let fetched_videos: Vec<Video> = Request::get("/tutorial/data.json")
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                videos.set(fetched_videos);
            });
            || ()
        });
    }

    let selected_video = use_state_eq(|| None);

    let on_video_select = {
        let selected_video = selected_video.clone();
        Callback::from(move |video: Video| selected_video.set(Some(video)))
    };

    let details = selected_video.as_ref().map(|video| {
        html! {
            <VideoDetails video={video.clone()} />
        }
    });

    html! {
        <>
            <h1>{ "RustConf Explorer" }</h1>
            <div>
                <h3>{"Videos to watch"}</h3>
                <VideoList videos={(*videos).clone()} on_click={on_video_select} />
            </div>
            <div>
                { for details }
            </div>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
