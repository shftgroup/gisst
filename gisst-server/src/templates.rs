

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

pub const UPLOAD_TEMPLATE: &'static str = r#"
<!doctype html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <title>test upload</title>
</head>
<body>
<form action="/content" method="post" enctype="multipart/form-data">
    Title: <input type="text" name="title"><br/>
    Version: <input type="text" name="version"><br/>
    <label for="content"> Choose a file to upload: </label>
    <input type="file" id="content" name="content">
    <button type="submit">Submit</button>
</form>
</body>
</html>
"#;
