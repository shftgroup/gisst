export interface Environment {
    environment_id: string,
    environment_name: string,
    environment_framework: string,
    environment_core_name: string,
    environment_core_version: string,
    environment_derived_from: string,
    environment_config: JSON
}

export interface Work {
    work_id: string,
    work_name: string,
    work_version: string,
    work_platform: string,
    created_on: Date
}

export interface Save {
    save_id: string,
    instance_id: string,
    save_short_desc: string,
    save_description: string,
    file_id: string,
    creator_id: string,
    created_on: Date
}
export interface State {
    state_id: string,
    instance_id: string,
    is_checkpoint: boolean,
    file_id: string,
    state_name: string,
    state_description: string,
    screenshot_id: string,
    replay_id: string,
    creator_id: string,
    state_derived_from: string,
    created_on: Date
}

export interface Replay {
    replay_id: string,
    instance_id: string,
    creator_id: string,
    replay_forked_from: string,
    file_id: string,
    created_on: Date
}
export interface Instance {
    instance_id: string,
    environment_id: string,
    work_id: string,
    instance_config: JSON,
    created_on: Date
}

export interface FullInstance {
    info: Instance,
    work: Work,
    states: State[],
    replays: Replay[],
    saves: Save[]
}

export class Convert {
    public static toInstance(json: string): Instance {
        return cast(JSON.parse(json), r("Instance"));
    }

    public static toEnvironment(json: string): Environment {
        return cast(JSON.parse(json), r("Environment"));
    }
    public static toReplay(json: string): Replay {
        return cast(JSON.parse(json), r("Replay"));
    }
    public static toState(json: string): State {
        return cast(JSON.parse(json), r("State"));
    }

    public static toSave(json: string): Save {
        return cast(JSON.parse(json), r("Save"));
    }
    public static toFullInstance(json: string): FullInstance {
        return cast(JSON.parse(json), r("FullInstance"));
    }
    public static instanceToJson(value: Instance): string {
        return JSON.stringify(uncast(value, r("Instance")), null, 2);
    }
    public static fullInstanceToJson(value: FullInstance): string {
        return JSON.stringify(uncast(value, r("FullInstance")), null, 2);
    }
    public static environmentToJson(value: Environment): string {
        return JSON.stringify(uncast(value, r("Environment")), null, 2);
    }
    public static replayToJson(value: Replay): string {
        return JSON.stringify(uncast(value, r("Replay")), null, 2);
    }
    public static saveToJson(value: Save): string {
        return JSON.stringify(uncast(value, r("Save")), null, 2);
    }
    public static stateToJson(value: State): string {
        return JSON.stringify(uncast(value, r("State")), null, 2);
    }
}

const typeMap: any = {
    "Environment": o([
        {json: "environment_id", js: "environment_id", typ: ""},
        {json: "environment_name", js: "environment_name", typ: ""},
        {json: "environment_framework", js: "environment_framework", typ: ""},
        {json: "environment_core_name", js: "environment_core_name", typ: ""},
        {json: "environment_core_version", js: "environment_core_version", typ: ""},
        {json: "environment_derived_from", js: "environment_derived_from", typ: ""},
        {json: "environment_instance_config", js: "environment_instance_config", typ: r("JSON")},
        {json: "created_on", js: "created_on", typ: r("Date")},
    ], false),
    "FullInstance": o([
        {json: "info", js: "info", typ: r("Instance")},
        {json: "work", js: "work", typ: r("Work")},
        {json: "states", js: "states", typ: a(r("State"))},
        {json: "replays", js: "replays", typ: a(r("Replay"))},
        {json: "saves", js: "saves", typ: a(r("Save"))},
    ], false),
    "Instance": o([
        {json: "instance_id", js: "instance_id", typ: ""},
        {json: "environment_id", js: "environment_id", typ: ""},
        {json: "work_id", js: "work_id", typ: ""},
        {json: "instance_config", js: "instance_config", typ: r("JSON")},
        {json: "created_on", js: "created_on", typ: r("Date")},
    ], false),
    "Replay": o([
        {json: "replay_id", js: "replay_id", typ: ""},
        {json: "instance_id", js: "instance_id", typ: ""},
        {json: "creator_id", js: "creator_id", typ: ""},
        {json: "replay_forked_from", js: "replay_forked_from", typ: ""},
        {json: "file_id", js: "file_id", typ: ""},
        {json: "created_on", js: "created_on", typ: r("Date")},
    ], false),
    "Save": o([
        {json: "save_id", js: "save_id", typ: ""},
        {json: "instance_id", js: "instance_id", typ: ""},
        {json: "save_short_desc", js: "save_short_desc", typ: ""},
        {json: "save_description", js: "save_description", typ: ""},
        {json: "file_id", js: "file_id", typ: ""},
        {json: "creator_id", js: "creator_id", typ: ""},
        {json: "created_on", js: "created_on", typ: r("Date")},
    ], false),
    "State": o([
        {json: "state_id", js: "state_id", typ: ""},
        {json: "instance_id", js: "instance_id", typ: ""},
        {json: "is_checkpoint", js: "is_checkpoint", typ: u(undefined, true) },
        {json: "file_id", js: "file_id", typ: ""},
        {json: "state_name", js: "state_name", typ: ""},
        {json: "state_description", js: "state_description", typ: ""},
        {json: "screenshot_id", js: "screenshot_id", typ: ""},
        {json: "replay_id", js: "replay_id", typ: ""},
        {json: "creator_id", js: "creator_id", typ: ""},
        {json: "state_derived_from", js: "state_derived_from", typ: ""},
        {json: "created_on", js: "created_on", typ: r("Date")},

    ], false),
    "Work": o([
        {json: "work_id", js: "work_id", typ: ""},
        {json: "work_name", js: "work_name", typ: ""},
        {json: "work_version", js: "work_version", typ: ""},
        {json: "work_platform", js: "work_platform", typ: ""},
        {json: "created_on", js: "created_on", typ: r("Date")},
    ], false),
};

function invalidValue(typ: any, val: any, key: any, parent: any = ''): never {
    const prettyTyp = prettyTypeName(typ);
    const parentText = parent ? ` on ${parent}` : '';
    const keyText = key ? ` for key "${key}"` : '';
    throw Error(`Invalid value${keyText}${parentText}. Expected ${prettyTyp} but got ${JSON.stringify(val)}`);
}

function prettyTypeName(typ: any): string {
    if (Array.isArray(typ)) {
        if (typ.length === 2 && typ[0] === undefined) {
            return `an optional ${prettyTypeName(typ[1])}`;
        } else {
            return `one of [${typ.map(a => { return prettyTypeName(a); }).join(", ")}]`;
        }
    } else if (typeof typ === "object" && typ.literal !== undefined) {
        return typ.literal;
    } else {
        return typeof typ;
    }
}

function jsonToJSProps(typ: any): any {
    if (typ.jsonToJS === undefined) {
        const map: any = {};
        typ.props.forEach((p: any) => map[p.json] = { key: p.js, typ: p.typ });
        typ.jsonToJS = map;
    }
    return typ.jsonToJS;
}

function jsToJSONProps(typ: any): any {
    if (typ.jsToJSON === undefined) {
        const map: any = {};
        typ.props.forEach((p: any) => map[p.js] = { key: p.json, typ: p.typ });
        typ.jsToJSON = map;
    }
    return typ.jsToJSON;
}

function transform(val: any, typ: any, getProps: any, key: any = '', parent: any = ''): any {
    function transformPrimitive(typ: string, val: any): any {
        if (typeof typ === typeof val) return val;
        return invalidValue(typ, val, key, parent);
    }

    function transformUnion(typs: any[], val: any): any {
        // val must validate against one typ in typs
        const l = typs.length;
        for (let i = 0; i < l; i++) {
            const typ = typs[i];
            try {
                return transform(val, typ, getProps);
            } catch (_) {}
        }
        return invalidValue(typs, val, key, parent);
    }

    function transformEnum(cases: string[], val: any): any {
        if (cases.indexOf(val) !== -1) return val;
        return invalidValue(cases.map(a => { return l(a); }), val, key, parent);
    }

    function transformArray(typ: any, val: any): any {
        // val must be an array with no invalid elements
        if (!Array.isArray(val)) return invalidValue(l("array"), val, key, parent);
        return val.map(el => transform(el, typ, getProps));
    }

    function transformDate(val: any): any {
        if (val === null) {
            return null;
        }
        const d = new Date(val);
        if (isNaN(d.valueOf())) {
            return invalidValue(l("Date"), val, key, parent);
        }
        return d;
    }

    function transformObject(props: { [k: string]: any }, additional: any, val: any): any {
        if (val === null || typeof val !== "object" || Array.isArray(val)) {
            return invalidValue(l(ref || "object"), val, key, parent);
        }
        const result: any = {};
        Object.getOwnPropertyNames(props).forEach(key => {
            const prop = props[key];
            const v = Object.prototype.hasOwnProperty.call(val, key) ? val[key] : undefined;
            result[prop.key] = transform(v, prop.typ, getProps, key, ref);
        });
        Object.getOwnPropertyNames(val).forEach(key => {
            if (!Object.prototype.hasOwnProperty.call(props, key)) {
                result[key] = transform(val[key], additional, getProps, key, ref);
            }
        });
        return result;
    }

    if (typ === "any") return val;
    if (typ === null) {
        if (val === null) return val;
        return invalidValue(typ, val, key, parent);
    }
    if (typ === false) return invalidValue(typ, val, key, parent);
    let ref: any = undefined;
    while (typeof typ === "object" && typ.ref !== undefined) {
        ref = typ.ref;
        typ = typeMap[typ.ref];
    }
    if (Array.isArray(typ)) return transformEnum(typ, val);
    if (typeof typ === "object") {
        return typ.hasOwnProperty("unionMembers") ? transformUnion(typ.unionMembers, val)
            : typ.hasOwnProperty("arrayItems")    ? transformArray(typ.arrayItems, val)
                : typ.hasOwnProperty("props")         ? transformObject(getProps(typ), typ.additional, val)
                    : invalidValue(typ, val, key, parent);
    }
    // Numbers can be parsed by Date but shouldn't be.
    if (typ === Date && typeof val !== "number") return transformDate(val);
    return transformPrimitive(typ, val);
}

function cast<T>(val: any, typ: any): T {
    return transform(val, typ, jsonToJSProps);
}

function uncast<T>(val: T, typ: any): any {
    return transform(val, typ, jsToJSONProps);
}

function l(typ: any) {
    return { literal: typ };
}

function a(typ: any) {
    return { arrayItems: typ };
}

function u(...typs: any[]) {
    return { unionMembers: typs };
}

function o(props: any[], additional: any) {
    return { props, additional };
}

function m(additional: any) {
    return { props: [], additional };
}

function r(name: string) {
    return { ref: name };
}