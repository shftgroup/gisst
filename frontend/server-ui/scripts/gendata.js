import { MeiliSearch } from "meilisearch";

console.log("Please make sure a meilisearch instance is running at :7701");
console.log("e.g.: cd backend/meili; ./meilisearch --no-analytics --master-key test-api-key --env development --db-path ./test --http-addr localhost:7701");
console.log("You also need to initialize the indices before running this script, e.g.:");
console.log(`
  cd backend; MEILI_URL=http://localhost:7701 MEILI_API_KEY=test-api-key cargo run --bin gisst-cli -- init-indices
`);

const client = new MeiliSearch({
  host: process.argv[2] ?? "http://localhost:7701",
  apiKey: "test-api-key-github-actions",
});

const instances = client.index('instance');
const instance_data = [
      { work_id:0, work_name:"work a", work_version:"0.1p", work_platform:"Sony Playstation",
        instance_id:0, row_num:0},
      { work_id:0, work_name:"work a", work_version:"0.1n", work_platform:"Nintendo N64",
        instance_id:1, row_num:1},
      { work_id:1, work_name:"work b",  work_version:"0.1", work_platform:"Nintendo Entertainment System",
        instance_id:2, row_num:2},
      { work_id:2, work_name:"work c",  work_version:"0.1", work_platform:"Super Nintendo Entertainment System",
        instance_id:3, row_num:3},
      { work_id:3, work_name:"work d",  work_version:"0.1", work_platform:"Nintendo Game Boy",
        instance_id:4, row_num:4},
      { work_id:4, work_name:"work e",  work_version:"0.1", work_platform:"Sega Genesis",
        instance_id:5, row_num:5},
      { work_id:5, work_name:"work f",  work_version:"0.1", work_platform:"Sega Master System",
        instance_id:6, row_num:6},
      { work_id:6, work_name:"work g",  work_version:"0.1", work_platform:"Sega Game Gear",
        instance_id:7, row_num:7},
      { work_id:7, work_name:"work h",  work_version:"0.1", work_platform:"Sony Playstation 2",
        instance_id:8, row_num:8},
      { work_id:8, work_name:"work i",  work_version:"0.1", work_platform:"Nintendo GameCube",
        instance_id:9, row_num:9},
      { work_id:9, work_name:"work j",  work_version:"0.1", work_platform:"Atari 2600",
        instance_id:10, row_num:10},
      {work_id:10, work_name:"work k",  work_version:"0.1", work_platform:"Intellivision",
        instance_id:11, row_num:11},
];
await instances.addDocuments(instance_data);

const states = client.index('state');
const state_data = [
      {work_id:0, work_name:"work a", work_version:"0.1p", work_platform:"Sony Playstation", instance_id:0,
       state_id:0, state_name:"first state", state_description:"the first obstacle in level 1-2", screenshot_id:0, file_id:0,
       created_on: 5, creator_id:0, creator_username: "user0", creator_full_name: "user zero"
      },
      {work_id:0, work_name:"work a", work_version:"0.1p", work_platform:"Sony Playstation", instance_id:0,
       state_id:1, state_name:"second state", state_description:"the end of level 3-1", screenshot_id:1, file_id:1,
       created_on: 0, creator_id:0, creator_username: "user0", creator_full_name: "user zero"
      },
      {work_id:0, work_name:"work a", work_version:"0.1p", work_platform:"Sony Playstation", instance_id:0,
       state_id:2, state_name:"third state", state_description:"the beginning of level 1-3", screenshot_id:2, file_id:2,
       created_on: 3, creator_id:1, creator_username: "user1", creator_full_name: "user one"
      },

      {work_id:0, work_name:"work a", work_version:"0.1n", work_platform:"Nintendo N64", instance_id:1,
       state_id:3, state_name:"first state n", state_description:"the first obstacle in level 1-2 n", screenshot_id:3, file_id:3,
       created_on: 7, creator_id:0, creator_username: "user0", creator_full_name: "user zero"
      },
      {work_id:0, work_name:"work a", work_version:"0.1n", work_platform:"Nintendo N64", instance_id:1,
       state_id:4, state_name:"second state n", state_description:"the end of level 3-1 n", screenshot_id:4, file_id:4,
       created_on: 11, creator_id:0, creator_username: "user0", creator_full_name: "user zero"
      },
      {work_id:0, work_name:"work a", work_version:"0.1n", work_platform:"Nintendo N64", instance_id:1,
       state_id:5, state_name:"third state n", state_description:"the beginning of level 1-3 n", screenshot_id:5, file_id:5,
       created_on: 4, creator_id:1, creator_username: "user1", creator_full_name: "user one"
      },

      {work_id:1, work_name:"work b", work_version:"0.1", work_platform:"Nintendo N64", instance_id:2,
       state_id:6, state_name:"work b state", state_description:"yet another state", screenshot_id:6, file_id:6,
       created_on: 12, creator_id:1, creator_username: "user1", creator_full_name: "user one"
      },
      {work_id:1, work_name:"work b", work_version:"0.1", work_platform:"Nintendo N64", instance_id:2,
       state_id:7, state_name:"work b state", state_description:"even yet another state", screenshot_id:7, file_id:7,
       created_on: 13, creator_id:0, creator_username: "user0", creator_full_name: "user zero"
      },
];
await states.addDocuments(state_data);

const replays = client.index('replay');
const replay_data = [
      {work_id:0, work_name:"work a", work_version:"0.1p", work_platform:"Sony Playstation", instance_id:0,
       replay_id:0, replay_name:"first replay", replay_description:"playing in level 1-2", file_id:0,
       created_on: 5, creator_id:0, creator_username: "user0", creator_full_name: "user zero"
      },
      {work_id:0, work_name:"work a", work_version:"0.1p", work_platform:"Sony Playstation", instance_id:0,
       replay_id:1, replay_name:"second replay", replay_description:"beating level 3-1", file_id:1,
       created_on: 0, creator_id:0, creator_username: "user0", creator_full_name: "user zero"
      },
      {work_id:0, work_name:"work a", work_version:"0.1p", work_platform:"Sony Playstation", instance_id:0,
       replay_id:2, replay_name:"third replay", replay_description:"starting level 1-3", file_id:2,
       created_on: 3, creator_id:1, creator_username: "user1", creator_full_name: "user one"
      },

      {work_id:0, work_name:"work a", work_version:"0.1n", work_platform:"Nintendo N64", instance_id:1,
       replay_id:3, replay_name:"first replay n", replay_description:"playing in level 1-2 n", file_id:3,
       created_on: 7, creator_id:0, creator_username: "user0", creator_full_name: "user zero"
      },
      {work_id:0, work_name:"work a", work_version:"0.1n", work_platform:"Nintendo N64", instance_id:1,
       replay_id:4, replay_name:"second replay n", replay_description:"beating level 3-1 n", file_id:4,
       created_on: 11, creator_id:0, creator_username: "user0", creator_full_name: "user zero"
      },
      {work_id:0, work_name:"work a", work_version:"0.1n", work_platform:"Nintendo N64", instance_id:1,
       replay_id:5, replay_name:"third replay n", replay_description:"starting level 1-3 n", file_id:5,
       created_on: 4, creator_id:1, creator_username: "user1", creator_full_name: "user one"
      },

      {work_id:1, work_name:"work b", work_version:"0.1", work_platform:"Nintendo N64", instance_id:2,
       replay_id:6, replay_name:"work b replay", replay_description:"yet another replay", file_id:6,
       created_on: 12, creator_id:1, creator_username: "user1", creator_full_name: "user one"
      },
      {work_id:1, work_name:"work b", work_version:"0.1", work_platform:"Nintendo N64", instance_id:2,
       replay_id:7, replay_name:"work b replay", replay_description:"even yet another replay", file_id:7,
       created_on: 13, creator_id:0, creator_username: "user0", creator_full_name: "user zero"
      },
];
await replays.addDocuments(replay_data);

const saves = client.index('save');
const save_data = [
      {work_id:0, work_name:"work a", work_version:"0.1p", work_platform:"Sony Playstation", instance_id:0,
       save_id:0, save_short_desc:"first save", save_description:"save at beginning of level 1-2", file_id:0,
       created_on: 5, creator_id:0, creator_username: "user0", creator_full_name: "user zero"
      },
      {work_id:0, work_name:"work a", work_version:"0.1p", work_platform:"Sony Playstation", instance_id:0,
       save_id:1, save_short_desc:"second save", save_description:"save at beginning of 3-1", file_id:1,
       created_on: 0, creator_id:0, creator_username: "user0", creator_full_name: "user zero"
      },
      {work_id:0, work_name:"work a", work_version:"0.1p", work_platform:"Sony Playstation", instance_id:0,
       save_id:2, save_short_desc:"third save", save_description:"save at beginning of 1-3", file_id:2,
       created_on: 3, creator_id:1, creator_username: "user1", creator_full_name: "user one"
      },

      {work_id:0, work_name:"work a", work_version:"0.1n", work_platform:"Nintendo N64", instance_id:1,
       save_id:3, save_short_desc:"first save n", save_description:"save at beginning of 1-2 n", file_id:3,
       created_on: 7, creator_id:0, creator_username: "user0", creator_full_name: "user zero"
      },
      {work_id:0, work_name:"work a", work_version:"0.1n", work_platform:"Nintendo N64", instance_id:1,
       save_id:4, save_short_desc:"second save n", save_description:"save at beginning of 3-1 n", file_id:4,
       created_on: 11, creator_id:0, creator_username: "user0", creator_full_name: "user zero"
      },
      {work_id:0, work_name:"work a", work_version:"0.1n", work_platform:"Nintendo N64", instance_id:1,
       save_id:5, save_short_desc:"third save n", save_description:"save at beginning of 1-3 n", file_id:5,
       created_on: 4, creator_id:1, creator_username: "user1", creator_full_name: "user one"
      },

      {work_id:1, work_name:"work b", work_version:"0.1", work_platform:"Nintendo N64", instance_id:2,
       save_id:6, save_short_desc:"work b save", save_description:"a save in work b", file_id:6,
       created_on: 12, creator_id:1, creator_username: "user1", creator_full_name: "user one"
      },
      {work_id:1, work_name:"work b", work_version:"0.1", work_platform:"Nintendo N64", instance_id:2,
       save_id:7, save_short_desc:"work b save", save_description:"another save in work b", file_id:7,
       created_on: 13, creator_id:0, creator_username: "user0", creator_full_name: "user zero"
      },
];
await saves.addDocuments(save_data);
