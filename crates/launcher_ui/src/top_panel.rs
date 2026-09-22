use bevy::prelude::*;
use bevy::text::FontSourceTemplate;

pub fn top_panel() -> impl Scene {
    let banner_aspect_ratio = Some(2560. / 640.);
    let icon_aspect_ratio = Some(159. / 121.);
    let banner_padding = UiRect::horizontal(Val::Px(2.)).with_top(Val::Px(5.));

    bsn! {
        Node {
            width: Val::Percent(100.0),
            height: Val::Vh(25.),
            align_items: AlignItems::Center,
            overflow: Overflow::hidden(),
            border_radius: BorderRadius::all(Val::Px(10.))
        }
        Children [
            (
                ImageNode {
                    image: "images/banner.png",
                }
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.),
                    aspect_ratio: banner_aspect_ratio,
                    padding: banner_padding,
                    border_radius: BorderRadius::all(Val::Px(10.))
                }
            ),
            (
                ImageNode {
                    image: "images/lucid_white_icon.png",
                }
                Node {
                    margin: UiRect::left(Val::Px(5.)),
                    height: Val::Percent(80.),
                    aspect_ratio: icon_aspect_ratio
                }
            ),
            (
                Text::new("Lucid Engine")
                TextColor(Color::linear_rgb(10., 10., 10.))
                TextFont {
                    font: FontSourceTemplate::Handle("fonts/montserrat.ttf"),
                    font_size: FontSize::Vw(5.),
                    weight: FontWeight(700)
                }
                Node {
                    padding: UiRect::all(Val::Px(5.))
                }
            )
        ]
    }
}
