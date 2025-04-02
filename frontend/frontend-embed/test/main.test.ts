import { it, vi, expect, Mock, describe } from "vitest";
import {fetchConfig, parseGisstUrl} from "../src/main"

const mock_gisst_localhost_url = "https://localhost:3000/00000000000000000000000000001036";
const mock_gisst_http_url = "https://gisst.pomona.edu/play/5fbf545e-a59f-4556-8626-7bcbb1effd80";
const mock_embed_config = {
    "instance": {
        "instance_id": "00000000-0000-0000-0000-000000001036",
        "work_id": "00000000-0000-0000-0000-000000001036",
        "environment_id": "00000000-0000-0000-0000-000000000002",
        "instance_config": null,
        "created_on": "2025-03-07T22:11:02.682433Z",
        "derived_from_instance": null,
        "derived_from_state": null
    },
    "work": {
        "work_id": "00000000-0000-0000-0000-000000001036",
        "work_name": "Snake",
        "work_version": "FreeDOS",
        "work_platform": "FreeDOS",
        "created_on": "2025-03-07T22:11:02.350106Z",
        "work_derived_from": null
    },
    "environment": {
        "environment_id": "00000000-0000-0000-0000-000000000002",
        "environment_name": "FreeDOS",
        "environment_framework": "v86",
        "environment_core_name": "v86",
        "environment_core_version": "0.1.0",
        "environment_derived_from": null,
        "environment_config": {
            "bios": {
                "url": "seabios.bin"
            },
            "fda": {
                "async": true,
                "fixed_chunk_size": 44194304,
                "url": "storage/c\\a\\a\\8\\07cd9656778aa01e7f99f37e31b76e24-freedos722.img"
            },
            "memory_size": 16777216,
            "vga_bios": {
                "url": "vgabios.bin"
            }
        },
        "created_on": "2025-03-07T20:40:28.308974Z"
    },
    "save": null,
    "start": {
        "type": "cold"
    },
    "manifest": [
        {
            "object_id": "00000000-0000-0000-0000-000000001036",
            "object_role": "content",
            "object_role_index": 0,
            "file_hash": "07cd9656778aa01e7f99f37e31b76e24",
            "file_filename": "freedos722.img",
            "file_source_path": "",
            "file_dest_path": "c\\a\\a\\8\\07cd9656778aa01e7f99f37e31b76e24-freedos722.img"
        }
    ],
    "host_url": "localhost:3000",
    "host_protocol": "https:",
    "citation_data": {
        "website_title": "",
        "url": "https://localhost:3000/data/00000000000000000000000000001036",
        "gs_page_view_date": "2025, March 13",
        "mla_page_view_date": "13 Mar. 2025",
        "bibtex_page_view_date": "2025-03-13",
        "site_published_year": "2025"
    }
}

//TESTS

describe("frontend-embed", () => {
  it("should parse localhost url", async () => {
    const [proto, root, query] = parseGisstUrl(mock_gisst_localhost_url);

    const fetchUrl = proto+"://"+root+"/data/"+query

    expect(proto).toEqual("https");
    expect(root).toEqual("localhost:3000")
    expect(query).toEqual("00000000000000000000000000001036")
    expect(fetchUrl).toEqual("https://localhost:3000/data/00000000000000000000000000001036")
  })

  it("should parse http url", async () => {
    const [proto, root, query] = parseGisstUrl(mock_gisst_http_url);

    expect(proto).toEqual("https");
    expect(root).toEqual("gisst.pomona.edu")
    expect(query).toEqual("5fbf545e-a59f-4556-8626-7bcbb1effd80")
  })

  it("should fetch an embed from server", async () => {
    global.fetch = vi.fn(() =>
                Promise.resolve({
                    json: () => Promise.resolve(mock_embed_config),
                }),
            ) as Mock;

    const config = await fetchConfig(mock_gisst_localhost_url)
    expect(config).toEqual(mock_embed_config)
  })

  it("should throw error when server response status is 500", async () => {
    global.fetch = vi.fn(() =>
                Promise.resolve({
                    status: 500,
                }),
            ) as Mock;
    await expect(fetchConfig(mock_gisst_localhost_url)).rejects.toThrow("Request Status 500: Internal Server Error")
  })

  it("should throw error when server response status is 500", async () => {
    global.fetch = vi.fn(() =>
                Promise.resolve({
                    status: 404,
                }),
            ) as Mock;
    await expect(fetchConfig(mock_gisst_localhost_url)).rejects.toThrow("Request Status 404: Instance Not Found")
  })
})
