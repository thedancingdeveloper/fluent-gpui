# Fluent 2 token audit

This table records the relationship between the Fluent 2 alias token groups and
`fluent-core`'s public token names. A `missing` entry means that the alias is
not currently represented by a dedicated `fluent-core` token; it is tracked by
a follow-up work item rather than silently substituted.

## Color aliases

| Fluent 2 alias group | Alias/state | `fluent-core` name | Notes |
| --- | --- | --- | --- |
| Neutral foreground | 1/all states | `ColorScheme::neutral_foreground_1.{default,hover,pressed,selected,disabled}` | Primary text/icon foreground |
| Neutral foreground | 2/all states | `ColorScheme::neutral_foreground_2.{default,hover,pressed,selected,disabled}` | Secondary text/icon foreground |
| Neutral foreground | 3/all states | `ColorScheme::neutral_foreground_3.{default,hover,pressed,selected,disabled}` | Tertiary text/icon foreground |
| Neutral background | 1/all states | `ColorScheme::neutral_background_1.{default,hover,pressed,selected,disabled}` | Page/window background |
| Neutral background | 2/all states | `ColorScheme::neutral_background_2.{default,hover,pressed,selected,disabled}` | Secondary/dim surface |
| Neutral background | 3/all states | `ColorScheme::neutral_background_3.{default,hover,pressed,selected,disabled}` | Neutral interactive fill |
| Neutral stroke | 1/all states | `ColorScheme::neutral_stroke_1.{default,hover,pressed,selected,disabled}` | Default border/outline |
| Neutral stroke | 2/all states | `ColorScheme::neutral_stroke_2.{default,hover,pressed,selected,disabled}` | Secondary/dimmer border |
| Neutral stroke | 3/all states | `ColorScheme::neutral_stroke_3.{default,hover,pressed,selected,disabled}` | Subtle divider/separator stroke |
| Brand | default/interactive | `ColorScheme::accent` | Brand/accent fill |
| Brand | hover | `ColorScheme::accent_hover` | Brand hover fill |
| Brand | pressed/selected | `ColorScheme::accent_selected` | Brand selected/pressed fill |
| Brand | disabled | `ColorScheme::accent_disabled` | Brand disabled fill |
| Brand foreground | default | `ColorScheme::on_neutral_accent` | Brand-colored text/icon |
| Status | info | `ColorScheme::status_info`, `status_info_bg`, `status_info_border` | Accent, background, and border |
| Status | success | `ColorScheme::status_success`, `status_success_bg`, `status_success_border` | Accent, background, and border |
| Status | warning | `ColorScheme::status_warning`, `status_warning_bg`, `status_warning_border` | Accent, background, and border |
| Status | error | `ColorScheme::status_error`, `status_error_bg`, `status_error_border` | Accent, background, and border |

## Effects and geometry aliases

| Fluent 2 alias group | Alias | `fluent-core` name |
| --- | --- | --- |
| Shadow | shadow2 | `ShadowTokens::shadow2` |
| Shadow | shadow4 | `ShadowTokens::shadow4` |
| Shadow | shadow8 | `ShadowTokens::shadow8` |
| Shadow | shadow16 | `ShadowTokens::shadow16` |
| Shadow | shadow28 | `ShadowTokens::shadow28` |
| Shadow | shadow64 | `ShadowTokens::shadow64` |
| Stroke width | thin | `StrokeWidthTokens::thin` |
| Stroke width | thick | `StrokeWidthTokens::thick` |
| Stroke width | thickest | `StrokeWidthTokens::thickest` |
| Corner radius | small/medium/large/pill | `RadiiTokens::sm`, `md`, `lg`, `pill` |

## Motion aliases

| Fluent 2 alias group | Alias | `fluent-core` name |
| --- | --- | --- |
| Curve | accelerate | `MotionTokens::accelerate` |
| Curve | decelerate | `MotionTokens::decelerate` |
| Curve | easy-ease | `MotionTokens::easy_ease` |
| Duration | ultra-fast | `MotionTokens::ultra_fast_duration_ms` |
| Duration | fast | `MotionTokens::fast_duration_ms` |
| Duration | normal | `MotionTokens::normal_duration_ms` |
| Duration | slow | `MotionTokens::slow_duration_ms` |
| Duration | ultra-slow | `MotionTokens::ultra_slow_duration_ms` |
| Popup duration | enter | `MotionTokens::popup_enter_duration_ms` |
| Popup duration | exit | `MotionTokens::popup_exit_duration_ms` |
| Popup offset | default | `MotionTokens::popup_offset` |
| Popup scale | default | `MotionTokens::popup_scale_delta` |

The audit intentionally distinguishes a semantic approximation from a complete
alias family. Existing names are useful compatibility points, but callers
should not infer missing state-specific aliases from them.
