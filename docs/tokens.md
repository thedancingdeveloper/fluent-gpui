# Fluent 2 token audit

This table records the relationship between the Fluent 2 alias token groups and
`fluent-core`'s public token names. A `missing` entry means that the alias is
not currently represented by a dedicated `fluent-core` token; it is tracked by
a follow-up work item rather than silently substituted.

## Color aliases

| Fluent 2 alias group | Alias/state | `fluent-core` name | Notes |
| --- | --- | --- | --- |
| Neutral foreground | 1/default | `ColorScheme::on_neutral` | Primary text/icon foreground |
| Neutral foreground | 1/hover | `missing` | No dedicated hover foreground |
| Neutral foreground | 1/pressed | `missing` | No dedicated pressed foreground |
| Neutral foreground | 1/selected | `ColorScheme::on_neutral_selected` | Selected text/icon foreground |
| Neutral foreground | 1/disabled | `ColorScheme::on_neutral_disabled` | Disabled text/icon foreground |
| Neutral foreground | 2/default | `ColorScheme::on_subtle` | Secondary text/icon foreground |
| Neutral foreground | 2/hover | `missing` | No dedicated hover foreground |
| Neutral foreground | 2/pressed | `missing` | No dedicated pressed foreground |
| Neutral foreground | 2/selected | `ColorScheme::on_subtle_selected` | Selected secondary foreground |
| Neutral foreground | 2/disabled | `ColorScheme::on_subtle_disabled` | Disabled secondary foreground |
| Neutral foreground | 3/default | `missing` | Tertiary foreground is not explicit |
| Neutral foreground | 3/hover | `missing` | Tertiary hover foreground is not explicit |
| Neutral foreground | 3/pressed | `missing` | Tertiary pressed foreground is not explicit |
| Neutral foreground | 3/selected | `missing` | Tertiary selected foreground is not explicit |
| Neutral foreground | 3/disabled | `missing` | Tertiary disabled foreground is not explicit |
| Neutral background | 1/default | `ColorScheme::surface` | Page/window background |
| Neutral background | 1/hover | `missing` | No dedicated background hover alias |
| Neutral background | 1/pressed | `missing` | No dedicated background pressed alias |
| Neutral background | 1/selected | `missing` | No dedicated background selected alias |
| Neutral background | 1/disabled | `missing` | No dedicated background disabled alias |
| Neutral background | 2/default | `ColorScheme::surface_dim` | Secondary/dim surface |
| Neutral background | 2/hover | `missing` | No dedicated background hover alias |
| Neutral background | 2/pressed | `missing` | No dedicated background pressed alias |
| Neutral background | 2/selected | `missing` | No dedicated background selected alias |
| Neutral background | 2/disabled | `missing` | No dedicated background disabled alias |
| Neutral background | 3/default | `ColorScheme::neutral` | Neutral interactive fill |
| Neutral background | 3/hover | `ColorScheme::neutral_hover` | Neutral interactive hover fill |
| Neutral background | 3/pressed | `ColorScheme::neutral_selected` | Used for selected/pressed fill |
| Neutral background | 3/selected | `ColorScheme::neutral_selected` | Neutral selected fill |
| Neutral background | 3/disabled | `ColorScheme::neutral_disabled` | Neutral disabled fill |
| Neutral stroke | 1/default | `ColorScheme::stroke_neutral` | Default border/outline |
| Neutral stroke | 1/hover | `missing` | No dedicated hover stroke |
| Neutral stroke | 1/pressed | `missing` | No dedicated pressed stroke |
| Neutral stroke | 1/selected | `missing` | No dedicated selected stroke |
| Neutral stroke | 1/disabled | `ColorScheme::stroke_neutral_disabled` | Disabled border |
| Neutral stroke | 2/default | `ColorScheme::stroke_neutral_dim` | Secondary/dimmer border |
| Neutral stroke | 2/hover | `missing` | No dedicated hover stroke |
| Neutral stroke | 2/pressed | `missing` | No dedicated pressed stroke |
| Neutral stroke | 2/selected | `missing` | No dedicated selected stroke |
| Neutral stroke | 2/disabled | `missing` | No dedicated disabled stroke |
| Neutral stroke | 3/default | `ColorScheme::stroke_neutral_subtle` | Subtle divider/separator stroke |
| Neutral stroke | 3/hover | `missing` | No dedicated hover stroke |
| Neutral stroke | 3/pressed | `missing` | No dedicated pressed stroke |
| Neutral stroke | 3/selected | `missing` | No dedicated selected stroke |
| Neutral stroke | 3/disabled | `missing` | No dedicated disabled stroke |
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
