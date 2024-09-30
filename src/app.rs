#[cfg(feature = "ssr")]
use crate::db;
use crate::error_template::{AppError, ErrorTemplate};
use crate::object_type::{DataType, ObjectType};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;
use strum::IntoEnumIterator;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/knowledge-base-app.css" />

        // sets the document title
        <Title text="Knowledge Base" />

        // content for this welcome page
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors /> }.into_view()
        }>
            <main>
                <Routes>
                    <Route path="" view=HomePage />
                </Routes>
            </main>
        </Router>
    }
}

#[server(GetObjectTypes, "/api")]
pub async fn get_object_types() -> Result<Vec<ObjectType>, ServerFnError> {
    match db::get_object_types().await {
        Ok(types) => Ok(types),
        Err(error) => {
            println!("Got error: {}", error);
            Err(ServerFnError::ServerError(error.to_string()))
        }
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    let dialog_ref: NodeRef<html::Dialog> = create_node_ref();
    let (count, set_count) = create_signal(1);
    let object_types = create_resource(|| (), |_| async { get_object_types().await });

    view! {
        <h1>"Library"</h1>
        <ObjectTypeDialog dialog_ref=dialog_ref count=count set_count=set_count />
        <Suspense fallback=move || view! { <p>"Loading Object Types"</p> }>
            // handles the error from the resource
            <ErrorBoundary fallback=|_| {
                view! { <p>"Something went wrong"</p> }
            }>
                {move || {
                    object_types
                        .get()
                        .map(move |result| {
                            result
                                .map(move |types| {
                                    view! {
                                        // the resource has a result
                                        // successful call from the server fn
                                        {types}
                                    }
                                })
                        })
                }}
            </ErrorBoundary>
        </Suspense>
        <button on:click=move |_| {
            let _ = dialog_ref.get().unwrap().show_modal();
        }>
            <h1>Add</h1>
        </button>
    }
}

#[component]
fn ObjectTypeDialog(
    dialog_ref: NodeRef<html::Dialog>,
    count: ReadSignal<i32>,
    set_count: WriteSignal<i32>,
) -> impl IntoView {
    let datatypes = DataType::iter()
        .map(|d| format!("{:?}", d))
        .collect::<Vec<String>>();
    view! {
        <dialog node_ref=dialog_ref class="modal">
            <div>
                <form>
                    <input name="name" type="text" placeholder="Name" />
                    <div>
                        {move || {
                            let mut rows = vec![];
                            for index in 0..count.get() {
                                let types = datatypes.clone();
                                rows.push(view! { <AttributeRow datatypes=types index=index /> });
                            }
                            rows
                        }}
                    </div>
                    <div>
                        <button on:click=move |ev| {
                            ev.prevent_default();
                            set_count.update(|count: &mut i32| *count += 1);
                        }>Add Row</button>
                    </div>
                    <div>
                        <button on:click=move |ev| {
                            ev.prevent_default();
                            dialog_ref.get().unwrap().close();
                        }>Submit</button>
                    </div>
                </form>
            </div>
        </dialog>
    }
}

#[component]
fn AttributeRow(datatypes: Vec<String>, index: i32) -> impl IntoView {
    view! {
        <div>
            <input
                name=format!("attribute-name{}", index)
                type="text"
                placeholder="Attribute Name"
            />
            <select name=format!("datatype{}", index)>
                <option disabled=true selected=true>
                    "Attribute DataType"
                </option>
                {datatypes
                    .into_iter()
                    .map(|datatype| view! { <option>{datatype}</option> })
                    .collect::<Vec<_>>()}
            </select>
        </div>
    }
}

#[component]
fn PlusIcon() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            view_box="0 0 24 24"
            stroke_width="1.5"
            stroke="currentColor"
            class="size-6"
        >
            <path stroke_linecap="round" stroke_linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
        </svg>
    }
}
