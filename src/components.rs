use leptos::{html::Input, *};
use uuid::Uuid;

#[derive(Clone, Debug)]
struct DraggedCard {
    card_id: String,
    from_lane_id: String,
}

#[derive(Clone, Debug)]
struct DroppedCardData {
    card_id: String,
    from_lane_id: String,
    to_lane_id: String,
}

#[derive(Copy, Clone)]
struct ContextSetter(WriteSignal<Option<DraggedCard>>);

#[component]
pub fn Board(name: String) -> impl IntoView {
    let (dragged_card, set_dragged_card) = create_signal::<Option<DraggedCard>>(None);
    provide_context(dragged_card);
    provide_context(set_dragged_card);
    provide_context(ContextSetter(set_dragged_card));

    let (dropped_card, set_dropped_card) = create_signal::<Option<DroppedCardData>>(None);

    let (lanes, set_lanes) = create_signal(vec![
        Lane::new("todo".to_string(), "TODO"),
        Lane::new("inprogress".to_string(), "In Progress"),
    ]);

    create_effect(move |_| {
        if let Some(data) = dropped_card.get() {
            let lanes = lanes.get();
            let Some(source_lane) = lanes.iter().find(|lane| lane.id == data.from_lane_id) else {
                leptos::logging::error!("No source lane!");
                return;
            };

            let Some(target_lane) = lanes.iter().find(|lane| lane.id == data.to_lane_id) else {
                leptos::logging::error!("No target lane!");
                return;
            };

            leptos::logging::log!("Source lane: {source_lane:?}");
            leptos::logging::log!("Target lane: {target_lane:?}");

            let source_cards = source_lane.cards.get();
            let Some(card) = source_cards.iter().find(|c| c.id == data.card_id) else {
                leptos::logging::error!("No such card in the source lane!");
                return;
            };
            source_lane
                .cards
                .update(|source_cards| source_cards.retain(|c| c.id != card.id));

            leptos::logging::log!("Card: {card:?}");

            let mut new_card = card.clone();
            new_card.lane_id = data.to_lane_id;
            // Update the dragged in case the drag goes on - because then the from_lane_id has
            // changed!
            set_dragged_card.set(Some(DraggedCard {
                card_id: new_card.id.clone(),
                from_lane_id: new_card.lane_id.clone(),
            }));

            target_lane
                .cards
                .update(|target_cards| target_cards.push(new_card));

            set_dropped_card.set(None);
        }
    });

    view! {
        // Code smell - without tracking the signals here, the application doesn't work as intended
        // Try commenting out this div and see what happens - you cannot drag around the cards!
        <div class="flex justify-around bg-nord15 hidden">
            <div>{move || if let Some(card) = dragged_card.get() {
               format!("Dragging card: {card:?}")
            } else {"Not dragging any card".to_string()}}
            </div>
            <div>
            {move || if let Some(card) = dropped_card.get() {
               format!("Dropped card: {card:?}")
            } else {"No dropped card".to_string()}}
            </div>
        </div>
        <div class="flex justify-between bg-nord3 text-nord4 text-xl text-center font-bold pl-4 pt-3 rounded-tr-lg w-3/6">
            <h1>{name}</h1>
            <button
                class="bg-nord0 p-2 rounded shadow"
                on:click=move |_| {
                set_lanes.update(|lanes| {
                    use fake::Fake;
                    use fake::faker::name::raw::*;

                    use fake::locales::*;
                    let name: String = Name(EN).fake();
                    let id = Uuid::new_v4().to_string();
                    lanes.push(Lane::new(id, name));
                });
            }>
                "Add lane"
            </button>
        </div>
        <div class="flex">
            <For
                each={move || lanes.get()}
                key=|lane| lane.name.clone()
                let:lane
            >
                <Lane lane set_dropped_card/>
            </For>
        </div>
    }
}

#[derive(Clone, Debug)]
struct Lane {
    id: String,
    name: RwSignal<String>,
    cards: RwSignal<Vec<Card>>,
}

impl Lane {
    pub fn new(id: String, name: impl Into<String>) -> Self {
        let name = create_rw_signal(name.into());
        let cards = create_rw_signal(vec![]);
        Self { id, name, cards }
    }
}

#[derive(Clone, Debug)]
struct Card {
    lane_id: String,
    id: String,
    name: RwSignal<String>,
    description: RwSignal<String>,
}

impl Card {
    pub fn new(id: String, lane_id: String, name: impl Into<String>) -> Self {
        let name = create_rw_signal(name.into());
        let description = create_rw_signal(String::default());
        Self {
            lane_id,
            id,
            name,
            description,
        }
    }

    pub fn with_description(self, description: impl Into<String>) -> Self {
        self.description
            .update(|current| *current = description.into());
        self
    }
}

const ESCAPE_KEY: u32 = 27;
const ENTER_KEY: u32 = 13;

#[component]
fn Lane(lane: Lane, set_dropped_card: WriteSignal<Option<DroppedCardData>>) -> impl IntoView {
    let Lane { id, name, cards } = lane;
    let is_edit = create_rw_signal(false);
    let dragged_card =
        use_context::<ReadSignal<Option<DraggedCard>>>().expect("dragged_card to exist");
    let set_dragged_card =
        use_context::<WriteSignal<Option<DraggedCard>>>().expect("set_dragged_card to exist");
    let draggable_is_over = create_rw_signal(false);

    let id_2 = id.clone();
    let id_string = id.clone();
    let lane_id = id.clone();
    let name_input = NodeRef::<Input>::new();

    view! {
        <section
            class="flex-1 flex-col p-2 rounded shadow"
            class:bg-nord14={move || draggable_is_over.get() }
            on:dragover=move |event| {
                event.prevent_default();
                //leptos::logging::log!("ondragover lane {id} {name}: {event:?}!", name=name.get())
                draggable_is_over.set(true);
                if let Some(card) = dragged_card.get() {
                    leptos::logging::log!("have card: {card:?}");

                    set_dropped_card.set(Some(DroppedCardData {
                        card_id: card.card_id,
                        from_lane_id: card.from_lane_id,
                        to_lane_id: id_2.clone(),
                    }));
                }
            }
            on:dragleave=move |event| {
                event.prevent_default();
                draggable_is_over.set(false);
                //leptos::logging::log!("ondragleave lane {id} {name}: {event:?}!", name=name.get())
            }
            on:drop=move |event| {
                draggable_is_over.set(false);
                set_dragged_card.set(None);
                leptos::logging::log!("ondrop lane {id} {name}: {event:?}!", id=id_string, name=name.get());
            }
        >
                <header class="flex justify-between p-4">
                //<h4>lane id: {id.to_string()}</h4>
                {move || if is_edit.get() {
                    view! {
                        <input
                            class="text-xl font-bold mb-2 w-full"
                            node_ref=name_input
                            value=name
                            on:focusout=move |ev| {
                                name.set(event_target_value(&ev));
                                is_edit.set(false);
                            }
                            on:keyup={move |ev: web_sys::KeyboardEvent| {
                                let key_code = ev.key_code();
                                if key_code == ENTER_KEY {
                                    name.set(event_target_value(&ev));
                                    is_edit.set(false);
                                } else if key_code == ESCAPE_KEY {
                                    is_edit.set(false);
                                }
                            }}
                        />
                    }.into_any()
                } else {
                        view! {
                            <div>
                                <h2
                                    class="text-xl font-bold text-center text-nord0 mb-4"
                                    on:dblclick=move |_| is_edit.set(true)
                                >
                                    {name}
                                </h2>
                            </div>
                        }.into_any()
                }}
                <button
                    class="bg-nord0 text-nord4 text-center font-bold p-2 mb-2 rounded shadow"
                    on:click=move |_| {
                        use fake::Fake;
                        use fake::faker::name::raw::*;
                        use fake::faker::lorem::en::*;

                        use fake::locales::*;
                        let name: String = Name(EN).fake();
                        let description: String = Words(3..50).fake::<Vec<String>>().join(" ");

                        let card_id = Uuid::new_v4().to_string();
                        leptos::logging::log!("lane_id: {lane_id}");
                        let card = Card::new(card_id, lane_id.clone(), name).with_description(description);
                        cards.update(|cards| cards.push(card))
                    }
                >Add card</button>
                </header>
            <main class="flex-1 p-2 rounded shadow">
                <For
                    each={move || cards.get().clone()}
                    key=|card| card.name.clone()
                    let:card
                >
                    <Card card/>
                </For>
            </main>
        </section>
    }
}

#[component]
fn Card(card: Card) -> impl IntoView {
    let Card {
        name,
        description,
        id,
        lane_id,
    } = card;
    let set_dragged_card = use_context::<ContextSetter>().unwrap().0;

    let is_edit = create_rw_signal(false);
    let name_input = NodeRef::<Input>::new();
    let description_input = NodeRef::<Input>::new();

    let (id, _) = create_signal(id);
    let (lane_id, _) = create_signal(lane_id);

    let start_drag = move || {
        set_dragged_card.set(Some(DraggedCard {
            card_id: id.get().clone(),
            from_lane_id: lane_id.get().clone(),
        }))
    };

    view! {
        //<h3>lane id: {lane_id}</h3>
        {move || if is_edit.get() {
            view!{
                <div class="bg-nord10 p-4 rounded shadow mb-2">
                    <div class="flex gap-2">
                        <input
                            class="text-xl w-full"
                            node_ref=name_input
                            value=name
                        />
                        <span class="bg-nord14">{id}</span>
                    </div>
                    <input
                        class="text-xl w-full"
                        node_ref=description_input
                        value=description
                    />
                    <button on:click=move |_| {
                        name.set(name_input.get().unwrap().value());
                        description.set(description_input.get().unwrap().value());
                        is_edit.set(false);
                    }>Save</button>
                </div>
            }.into_any()
        } else {
            view!{
                <div class="bg-nord10 p-4 rounded shadow mb-2 overflow-hidden cursor-move"
                    on:dblclick=move |_| {
                        leptos::logging::log!("double clicked!");
                        is_edit.set(true)
                    }
                    draggable="true"
                    on:dragstart=move |_| {
                        leptos::logging::log!("starting to drag!");
                        start_drag();
                    }
                    on:drag=move |_| {
                        leptos::logging::log!("dragging card!")
                    }
                >
                    <div class="flex gap-2 overflow-hidden">
                        <h3 class="text-xl font-bold">{name}</h3>
                        <span class="bg-nord14 text-xs text-wrap">{id}</span>
                    </div>
                    <p class="text-wrap">{description}</p>
                </div>
            }.into_any()
        }}
    }
}
