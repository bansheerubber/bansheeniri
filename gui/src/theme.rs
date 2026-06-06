use iced::{
	border::Radius,
	widget::{
		button, container,
		scrollable::{self, AutoScroll},
	},
	Border, Color, Shadow, Theme, Vector,
};

pub fn scrollbar_visible(theme: &Theme) -> scrollable::Style {
	let rail = scrollable::Rail {
		background: None,
		border: Border::default(),
		scroller: scrollable::Scroller {
			background: theme.extended_palette().background.strongest.color.into(),
			border: Border {
				color: Color::TRANSPARENT,
				width: 0.0,
				radius: Radius::new(5),
			},
		},
	};

	return scrollable::Style {
		container: Color::TRANSPARENT.into(),
		vertical_rail: rail.clone(),
		horizontal_rail: rail,
		gap: None,
		auto_scroll: AutoScroll {
			background: Color::TRANSPARENT.into(),
			border: Border {
				color: Color::TRANSPARENT,
				width: 0.0,
				radius: Radius::new(0),
			},
			shadow: Shadow {
				color: Color::TRANSPARENT,
				offset: Vector { x: 0.0, y: 0.0 },
				blur_radius: 0.0,
			},
			icon: Color::TRANSPARENT,
		},
	};
}

pub fn scrollbar_invisible() -> scrollable::Style {
	let rail = scrollable::Rail {
		background: None,
		border: Border::default(),
		scroller: scrollable::Scroller {
			background: Color::TRANSPARENT.into(),
			border: Border {
				color: Color::TRANSPARENT,
				width: 0.0,
				radius: Radius::new(5),
			},
		},
	};

	return scrollable::Style {
		container: Color::TRANSPARENT.into(),
		vertical_rail: rail.clone(),
		horizontal_rail: rail,
		gap: None,
		auto_scroll: AutoScroll {
			background: Color::TRANSPARENT.into(),
			border: Border {
				color: Color::TRANSPARENT,
				width: 0.0,
				radius: Radius::new(0),
			},
			shadow: Shadow {
				color: Color::TRANSPARENT,
				offset: Vector { x: 0.0, y: 0.0 },
				blur_radius: 0.0,
			},
			icon: Color::TRANSPARENT,
		},
	};
}

pub fn invisible_button(theme: &Theme) -> button::Style {
	button::Style {
		background: None,
		text_color: theme.palette().text,
		border: Border::default(),
		shadow: Shadow::default(),
		snap: false,
	}
}

pub fn window(theme: &Theme) -> container::Style {
	container::Style {
		text_color: Some(theme.palette().text),
		background: Some(theme.extended_palette().background.strongest.color.into()),
		border: Border::default(),
		shadow: Shadow::default(),
		snap: false,
	}
}

pub fn selected_monitor(theme: &Theme) -> container::Style {
	container::Style {
		text_color: Some(theme.palette().text),
		background: Some(theme.extended_palette().background.strong.color.into()),
		border: Border::default()
			.rounded(2)
			.width(1)
			.color(theme.extended_palette().background.weakest.color),
		shadow: Shadow::default(),
		snap: false,
	}
}

pub fn monitor(theme: &Theme) -> container::Style {
	container::Style {
		text_color: Some(theme.palette().text),
		background: Some(theme.extended_palette().background.strong.color.into()),
		border: Border::default()
			.rounded(2)
			.width(1)
			.color(theme.extended_palette().background.strong.color),
		shadow: Shadow::default(),
		snap: false,
	}
}

pub fn selected_button(theme: &Theme) -> button::Style {
	button::Style {
		background: Some(theme.extended_palette().background.neutral.color.into()),
		text_color: theme.palette().text,
		border: Border::default(),
		shadow: Shadow::default(),
		snap: false,
	}
}
