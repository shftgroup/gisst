

pub const PLAYER_TEMPLATE: &'static str = r#"
<!doctype html>
<html lang="en">
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>GISST Player</title>
    <script type="module" crossorigin src="/assets/index.js"></script>
    <link rel="stylesheet" href="/assets/index.css">
    {% if player_params.platform.platform_framework == "retroarch" %}
    <script src="ra/libretro_adapter.js"></script>
    {% elif player_params.platform.platform_framework == "v86" %}
    <script src="v86/libv86.js"></script>
    {% endif %}
</head>

<body data-artifact-uuid="{{ player_params.content.content_id }}">
    <div id="ui"></div>
</body>
</html>
"#;
