//! A presentational card container with rounded corners, optional
//! background color or image, and two visual variants (flat with a
//! border, or elevated with a drop shadow).
//!
//! See [`Card`] for the widget and [`Catalog`]/[`Style`] for styling.

mod style;

use std::cell::{Cell, RefCell};
use std::marker::PhantomData;

use iced::advanced::image as advanced_image;
use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::svg as advanced_svg;
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::alignment::{self, Alignment};
use iced::mouse;
use iced::{
    Background, ContentFit, Element, Event, Length, Padding, Pixels, Point, Radians, Rectangle,
    Size, Vector,
};

pub use style::{Catalog, Style, StyleFn, Variant, default, elevated, flat};

use crate::{PaddingSource, SpacingBase};

/// A background image for a [`Card`], either raster or vector.
#[derive(Debug, Clone)]
enum BackgroundImage {
    Raster(advanced_image::Handle),
    Svg(advanced_svg::Handle),
}

/// A cached rasterization of an SVG background at a specific pixel
/// size.
#[derive(Debug, Clone)]
struct SvgRaster {
    /// Identifier of the source [`advanced_svg::Handle`] the raster
    /// was produced from.
    source_id: u64,
    /// Pixel size of the cached raster.
    size: Size<u32>,
    /// The decoded [`advanced_image::Handle`] ready to be drawn.
    handle: advanced_image::Handle,
}

/// Internal widget state for a [`Card`].
#[derive(Debug)]
struct CardState {
    /// Lazily-populated cache for the SVG background image, if any.
    /// Wrapped in a [`RefCell`] so that the immutable [`Widget::draw`]
    /// hook can refresh it when the card's size or SVG source
    /// changes.
    svg_raster: RefCell<Option<SvgRaster>>,
    /// Last-resolved padding, in logical pixels. Cached so that
    /// [`Widget::layout`], which has no access to the active theme,
    /// can use the most recent value seen by [`Widget::draw`]. The
    /// initial value resolves [`PaddingSource::Absolute`] against
    /// itself (theme-independent) or applies the default spacing base
    /// for [`PaddingSource::Space`] until the first draw refreshes it.
    padding_cache: Cell<Padding>,
}

impl Default for CardState {
    fn default() -> Self {
        Self {
            svg_raster: RefCell::new(None),
            padding_cache: Cell::new(Padding::new(0.0)),
        }
    }
}

/// A presentational container with rounded corners, an optional
/// background color or image, and two visual variants.
///
/// By default a [`Card`] is [`Variant::Flat`] — a bordered frame with
/// no shadow. Switch to a shadowed look with [`Card::elevated`].
///
/// # Roundness
///
/// The card's corner roundness is driven by the [`Style::border`]'s
/// [`Radius`](iced::border::Radius), which is resolved from the active
/// [`Catalog`]. Override [`Card::style`] (or supply your own class) to
/// change it.
///
/// # Backgrounds
///
/// A solid color or gradient can be set with [`Card::background`] and
/// is drawn first. On top of it, an optional raster or vector image —
/// configured via [`Card::background_image`] or
/// [`Card::background_svg`] — is drawn, clipped to the rounded bounds.
/// When both are set, the color acts as a backdrop under the image.
///
/// # Example
///
/// ```no_run
/// use iced::widget::text;
/// use iced_ui::{Card, Space, Theme};
///
/// # type Message = ();
/// # fn _build() -> iced::Element<'static, Message, Theme> {
/// Card::new(text("Hello!"))
///     .padding(Space::sx(4.0))
///     .elevated()
///     .into()
/// # }
/// ```
pub struct Card<'a, Message, Theme = crate::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer
        + advanced_image::Renderer<Handle = advanced_image::Handle>
        + advanced_svg::Renderer,
{
    content: Element<'a, Message, Theme, Renderer>,
    width: Length,
    height: Length,
    max_width: f32,
    max_height: f32,
    padding: PaddingSource,
    variant: Variant,
    background: Option<Background>,
    background_image: Option<BackgroundImage>,
    image_fit: ContentFit,
    roundness_override: Option<f32>,
    class: <Theme as Catalog>::Class<'a>,
    _renderer: PhantomData<Renderer>,
}

impl<'a, Message, Theme, Renderer> Card<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: renderer::Renderer
        + advanced_image::Renderer<Handle = advanced_image::Handle>
        + advanced_svg::Renderer,
{
    /// Creates a new [`Card`] wrapping the given content.
    pub fn new(content: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        let content = content.into();
        let size = content.as_widget().size_hint();

        Self {
            content,
            width: size.width.fluid(),
            height: size.height.fluid(),
            max_width: f32::INFINITY,
            max_height: f32::INFINITY,
            padding: PaddingSource::from(crate::Space::sx(2.0)),
            variant: Variant::Flat,
            background: None,
            background_image: None,
            image_fit: ContentFit::Cover,
            roundness_override: None,
            class: <Theme as Catalog>::default(),
            _renderer: PhantomData,
        }
    }

    /// Sets the width of the [`Card`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Card`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the maximum width of the [`Card`].
    pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
        self.max_width = max_width.into().0;
        self
    }

    /// Sets the maximum height of the [`Card`].
    pub fn max_height(mut self, max_height: impl Into<Pixels>) -> Self {
        self.max_height = max_height.into().0;
        self
    }

    /// Sets the inner padding of the [`Card`]. Defaults to
    /// [`Space::sx(2.0)`](crate::Space::sx), which resolves to
    /// `16.0` logical pixels at the default [`spacing`].
    ///
    /// Accepts a [`Space`](crate::Space), an [`iced::Padding`] or a
    /// raw `[f32; 2]` (absolute).
    ///
    /// [`spacing`]: crate::Theme::spacing
    pub fn padding(mut self, padding: impl Into<PaddingSource>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets an explicit background color (or gradient) for the
    /// [`Card`], overriding the one provided by its [`Style`].
    pub fn background(mut self, background: impl Into<Background>) -> Self {
        self.background = Some(background.into());
        self
    }

    /// Sets a raster background image for the [`Card`].
    ///
    /// The image is drawn on top of the background color and clipped
    /// to the card's rounded rectangle.
    pub fn background_image(mut self, handle: impl Into<advanced_image::Handle>) -> Self {
        self.background_image = Some(BackgroundImage::Raster(handle.into()));
        self
    }

    /// Sets a vector (SVG) background image for the [`Card`].
    ///
    /// The image is drawn on top of the background color and clipped
    /// to the card's rounded rectangle.
    pub fn background_svg(mut self, handle: impl Into<advanced_svg::Handle>) -> Self {
        self.background_image = Some(BackgroundImage::Svg(handle.into()));
        self
    }

    /// Sets how the background image should fit inside the [`Card`].
    /// Defaults to [`ContentFit::Cover`].
    pub fn image_fit(mut self, fit: ContentFit) -> Self {
        self.image_fit = fit;
        self
    }

    /// Overrides the corner roundness of the [`Card`] in logical
    /// pixels, bypassing the theme's roundness setting. Set to `0` for
    /// sharp corners.
    pub fn roundness(mut self, roundness: impl Into<Pixels>) -> Self {
        self.roundness_override = Some(roundness.into().0);
        self
    }

    /// Renders the [`Card`] as a flat, bordered frame. This is the
    /// default variant.
    pub fn flat(mut self) -> Self {
        self.variant = Variant::Flat;
        self
    }

    /// Renders the [`Card`] with a drop shadow.
    pub fn elevated(mut self) -> Self {
        self.variant = Variant::Elevated;
        self
    }
}

impl<'a, Message, Renderer> Card<'a, Message, crate::Theme, Renderer>
where
    Renderer: renderer::Renderer
        + advanced_image::Renderer<Handle = advanced_image::Handle>
        + advanced_svg::Renderer,
{
    /// Sets the style of the [`Card`] using a
    /// `Fn(&Theme, Variant) -> Style` closure.
    ///
    /// This is available when `Theme = `[`crate::Theme`].
    pub fn style(mut self, style: impl Fn(&crate::Theme, Variant) -> Style + 'a) -> Self {
        self.class = Box::new(style);
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Card<'a, Message, Theme, Renderer>
where
    Theme: Catalog + SpacingBase,
    Renderer: renderer::Renderer
        + advanced_image::Renderer<Handle = advanced_image::Handle>
        + advanced_svg::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<CardState>()
    }

    fn state(&self) -> tree::State {
        // Pre-seed the padding cache with the absolute resolution of
        // the configured padding source. For `Space` variants we use
        // the default spacing base so the very first layout produces
        // a sensible value; `draw` will refresh the cache against the
        // actual theme on every frame.
        let initial_padding = self.padding.resolve(crate::Theme::DEFAULT_SPACING);
        tree::State::new(CardState {
            svg_raster: RefCell::new(None),
            padding_cache: Cell::new(initial_padding),
        })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_ref::<CardState>();
        let padding = state.padding_cache.get();
        layout::positioned(
            &limits.max_width(self.max_width).max_height(self.max_height),
            self.width,
            self.height,
            padding,
            |limits| {
                self.content.as_widget_mut().layout(
                    &mut tree.children[0],
                    renderer,
                    &limits.loose(),
                )
            },
            |content, size| {
                content.align(
                    Alignment::from(alignment::Horizontal::Left),
                    Alignment::from(alignment::Vertical::Top),
                    size,
                )
            },
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        renderer_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let Some(clipped_viewport) = bounds.intersection(viewport) else {
            return;
        };

        let mut style = theme.style(&self.class, self.variant);

        // Apply the roundness override if set.
        if let Some(r) = self.roundness_override {
            style.border.radius = r.into();
        }

        // Refresh the padding cache against the active theme so the
        // next layout sees the most recent spacing base.
        let state = tree.state.downcast_ref::<CardState>();
        state
            .padding_cache
            .set(self.padding.resolve(theme.spacing()));

        // 1. Draw the frame (shadow + background color + border) as a
        //    single rounded quad. The background provided on the
        //    widget directly takes precedence over the style's.
        let background = self
            .background
            .or(style.background)
            .unwrap_or(Background::Color(iced::Color::TRANSPARENT));

        if self.background.is_some()
            || style.background.is_some()
            || style.border.width > 0.0
            || style.shadow.color.a > 0.0
        {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.border,
                    shadow: style.shadow,
                    ..renderer::Quad::default()
                },
                background,
            );
        }

        // 2. Draw the background image on top. Raster images go
        //    through `image::Renderer::draw_image`, which supports
        //    rounded-corner clipping via `border_radius`. SVG
        //    backgrounds are rasterized via `resvg` at the card's
        //    current pixel size and routed through the same raster
        //    path so they get the same rounded clipping (SVG's own
        //    draw path has no `border_radius`).
        if let Some(image) = &self.background_image {
            match image {
                BackgroundImage::Raster(handle) => {
                    let drawing_bounds =
                        raster_fit_bounds(renderer, handle, bounds, self.image_fit);
                    renderer.with_layer(bounds, |renderer| {
                        <Renderer as advanced_image::Renderer>::draw_image(
                            renderer,
                            advanced_image::Image {
                                handle: handle.clone(),
                                filter_method: advanced_image::FilterMethod::Linear,
                                rotation: Radians(0.0),
                                border_radius: style.border.radius,
                                opacity: 1.0,
                                snap: true,
                            },
                            drawing_bounds,
                            bounds,
                        );
                    });
                }
                BackgroundImage::Svg(handle) => {
                    if let Some(rasterized) = ensure_svg_raster(state, handle, bounds) {
                        <Renderer as advanced_image::Renderer>::draw_image(
                            renderer,
                            advanced_image::Image {
                                handle: rasterized,
                                filter_method: advanced_image::FilterMethod::Linear,
                                rotation: Radians(0.0),
                                border_radius: style.border.radius,
                                opacity: 1.0,
                                snap: true,
                            },
                            bounds,
                            bounds,
                        );
                    }
                }
            }
        }

        // 3. Draw the child content, clipped to the card's bounds.
        let content_layout = layout.children().next().unwrap();
        let child_style = renderer::Style {
            text_color: style.text_color.unwrap_or(renderer_style.text_color),
        };
        renderer.with_layer(clipped_viewport, |renderer| {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                &child_style,
                content_layout,
                cursor,
                &clipped_viewport,
            );
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

/// Computes the fitted drawing bounds for a raster background image
/// inside the card.
fn raster_fit_bounds<Renderer>(
    renderer: &Renderer,
    handle: &advanced_image::Handle,
    bounds: Rectangle,
    fit: ContentFit,
) -> Rectangle
where
    Renderer: advanced_image::Renderer<Handle = advanced_image::Handle>,
{
    let size =
        <Renderer as advanced_image::Renderer>::measure_image(renderer, handle).unwrap_or_default();
    let intrinsic = Size::new(size.width as f32, size.height as f32);
    let intrinsic = if intrinsic.width <= 0.0 || intrinsic.height <= 0.0 {
        bounds.size()
    } else {
        intrinsic
    };

    let fitted = fit.fit(intrinsic, bounds.size());
    let position = Point::new(
        bounds.center_x() - fitted.width / 2.0,
        bounds.center_y() - fitted.height / 2.0,
    );
    Rectangle::new(position, fitted)
}

/// Returns the cached raster [`advanced_image::Handle`] for the given
/// SVG background, rasterizing (or re-rasterizing) it through `resvg`
/// as needed so that the cache matches both the current SVG source
/// and the card's current pixel size.
fn ensure_svg_raster(
    state: &CardState,
    handle: &advanced_svg::Handle,
    bounds: Rectangle,
) -> Option<advanced_image::Handle> {
    let width = bounds.width.ceil().max(0.0) as u32;
    let height = bounds.height.ceil().max(0.0) as u32;
    if width == 0 || height == 0 {
        return None;
    }

    let source_id = handle.id();
    let target = Size::new(width, height);

    let mut slot = state.svg_raster.borrow_mut();
    if let Some(cached) = slot.as_ref()
        && cached.source_id == source_id
        && cached.size == target
    {
        return Some(cached.handle.clone());
    }

    let rasterized = rasterize_svg(handle, width, height)?;
    let image_handle = advanced_image::Handle::from_rgba(width, height, rasterized);
    *slot = Some(SvgRaster {
        source_id,
        size: target,
        handle: image_handle.clone(),
    });
    Some(image_handle)
}

/// Rasterizes the given SVG [`advanced_svg::Handle`] into a straight
/// (non-premultiplied) RGBA byte buffer of `width * height * 4`
/// pixels.
fn rasterize_svg(handle: &advanced_svg::Handle, width: u32, height: u32) -> Option<Vec<u8>> {
    let bytes = match handle.data() {
        advanced_svg::Data::Bytes(bytes) => bytes.clone().into_owned(),
        advanced_svg::Data::Path(path) => std::fs::read(path).ok()?,
    };

    let tree = usvg::Tree::from_data(&bytes, &usvg::Options::default()).ok()?;
    let tree_size = tree.size();
    if tree_size.width() <= 0.0 || tree_size.height() <= 0.0 {
        return None;
    }

    let sx = width as f32 / tree_size.width();
    let sy = height as f32 / tree_size.height();
    let transform = tiny_skia::Transform::from_scale(sx, sy);

    let mut pixmap = tiny_skia::Pixmap::new(width, height)?;
    resvg::render(&tree, transform, &mut pixmap.as_mut());

    // `tiny_skia` stores premultiplied RGBA; iced's `image::Handle`
    // expects straight (non-premultiplied) RGBA bytes.
    let mut pixels = Vec::with_capacity((width as usize) * (height as usize) * 4);
    for pixel in pixmap.pixels() {
        let c = pixel.demultiply();
        pixels.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    Some(pixels)
}

impl<'a, Message, Theme, Renderer> From<Card<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + SpacingBase + 'a,
    Renderer: renderer::Renderer
        + advanced_image::Renderer<Handle = advanced_image::Handle>
        + advanced_svg::Renderer
        + 'a,
{
    fn from(card: Card<'a, Message, Theme, Renderer>) -> Self {
        Element::new(card)
    }
}
