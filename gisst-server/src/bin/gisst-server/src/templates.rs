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
            let filename = fp.clone().split('.').collect::<Vec<_>>()[0].to_string();
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

pub const PLAYER_TEMPLATE: &str = r#"
<!doctype html>
<html lang="en">
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>GISST Player</title>
    <script type="module" crossorigin src="/assets/index.js"></script>
    <link rel="stylesheet" href="/assets/index.css">
    {% if player_params.environment.environment_framework == "retroarch" %}
    <script src="/ra/libretro_adapter.js"></script>
    {% elif player_params.environment.environment_framework == "v86" %}
    <script src="/v86/libv86.js"></script>
    {% endif %}
    <script id="config" type="application/json">
    {{ player_params|tojson }}
    </script>
</head>

<body>
      <div class="webplayer-container">
          <div class="webplayer_border text-xs-center" id="canvas_div">
            <div style="white-space: pre; font: 14px monospace; line-height: 14px" class="hidden" id="webplayer-textmode"></div>
            <canvas class="webplayer hidden" id="canvas" tabindex="1" oncontextmenu="event.preventDefault()"></canvas>
              <img id="webplayer-preview" class="webplayer-preview" src="/media/canvas.png" width="960px" height="720px" alt="Loading Icon">
          </div>
      </div>
      <ul id="v86_controls" class="hidden">
        <button type="button" id="v86_save">Save State</button>
        <button type="button" id="v86_record">Record Replay</button>
        <button type="button" id="v86_stop">Stop Replay</button>
      </ul>
    <div id="ui"></div>
</body>
</html>
"#;

#[allow(dead_code)]
pub const UPLOAD_TEMPLATE: &str = r#"
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
