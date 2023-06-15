use gisstlib::GISSTError;
use std::collections::HashMap;
use std::fs;

#[allow(dead_code)]
pub struct TemplateHandler {
    templates: HashMap<String, String>,
}

impl TemplateHandler {
    pub fn new(template_path: &str) -> Result<Self, GISSTError> {
        let path = std::env::current_dir()?;
        println!("The current directory is {}", path.display());
        // TODO: make this check for directory
        println!("{}", template_path);
        let template_names = fs::read_dir(template_path).unwrap();
        let mut templates = HashMap::new();

        for filepath in template_names {
            let fp = filepath.unwrap().file_name().to_string_lossy().to_string();
            let filename = fp.clone().split(".").collect::<Vec<_>>()[0].to_string();
            println!("Adding template: {} to Template Handler", &filename);
            templates.insert(
                filename,
                fs::read_to_string(format!("{}/{}", template_path, &fp))
                    .map_err(|_e| GISSTError::TemplateError)?,
            );
        }

        Ok(Self { templates })
    }

    // pub fn get_template(&self, template_name: &str) -> Result<&str, GISSTError> {
    //     match self.templates.get(template_name) {
    //         Some(template) => Ok(template),
    //         None => Err(GISSTError::TemplateError),
    //     }
    // }
}

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

#[allow(dead_code)]
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
