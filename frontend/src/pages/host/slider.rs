// This code is based on the example from Yew's repository: 
// https://github.com/yewstack/yew/blob/master/examples/boids/src/slider.rs


use std::cell::Cell;

use web_sys::HtmlInputElement;
use yew::events::InputEvent;
use yew::{html, Callback, Component, Context, Html, Properties, TargetCast};

thread_local! {
    static SLIDER_ID: Cell<usize> = Cell::default();
}
fn next_slider_id() -> usize {
    SLIDER_ID.with(|cell| cell.replace(cell.get() + 1))
}

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub label: &'static str,
    pub value: i32,
    pub onchange: Callback<i32>,
    #[prop_or_default]
    pub min: i32,
    pub max: i32,
    #[prop_or(1)]
    pub step: i32,
}

pub struct Slider {
    id: usize,
}
impl Component for Slider {
    type Message = ();
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            id: next_slider_id(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        unimplemented!()
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
       let props = ctx.props();
        
        let display_value = format!("{}", props.value);
        let id = format!("slider-{}", self.id);

        let oninput = props.onchange.reform(|e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            input.value_as_number() as i32
        });

        html! {
            <div class="slider">
                <label for={id.clone()} class="slider__label">{ props.label }</label>
                <input type="range"
                    value={props.value.to_string()}
                    {id}
                    class="slider__input"
                    min={props.min.to_string()} max={props.max.to_string()} step={props.step.to_string()}
                    {oninput}
                />
                <span class="slider__value">{ display_value }</span>
            </div>
        }
    }
}
