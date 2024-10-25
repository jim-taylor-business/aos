use crate::ui::components::common::icon::{
  Icon,
  IconType::{Eye, EyeSlash},
};
use leptos::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputType {
  Text,
  Password,
}

#[component]
pub fn TextInput(
  #[prop(optional)] disabled: MaybeProp<bool>,
  #[prop(optional)] required: MaybeProp<bool>,
  #[prop(into)] id: TextProp,
  #[prop(into)] name: TextProp,
  #[prop(into)] label: TextProp,
  #[prop(into)] on_input: Callback<String, ()>,
  #[prop(default = InputType::Text)] input_type: InputType,
  #[prop(optional)] validation_class: MaybeSignal<String>,
) -> impl IntoView {
  let show_password = RwSignal::new(false);
  let eye_icon = Signal::derive(move || with!(|show_password| if *show_password { EyeSlash } else { Eye }));

  view! {
    <label class="flex relative gap-2 items-center">
      <input
        type={move || { if input_type == InputType::Text || show_password.get() { "text" } else { "password" } }}
        id={id}
        class={move || { format!("input input-bordered p-4 grow {}", validation_class.get()) }}
        placeholder={move || label.get()}
        name={move || name.get()}
        disabled={move || disabled.get().unwrap_or(false)}
        required={move || required.get().unwrap_or(false)}
        on:input={move |e| {
          on_input.call(event_target_value(&e));
        }}
      />
      <Show when={move || input_type == InputType::Password}>
        <button
          type="button"
          class="absolute bottom-2 btn btn-ghost btn-sm btn-circle end-1 text-accent"
          on:click={move |_| update!(|show_password| *show_password = !*show_password)}
        >
          <Icon icon={eye_icon} />
        </button>
      </Show>
    </label>
  }
}
