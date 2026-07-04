export type UnknownRequestStatus = 'new' | 'ignored' | 'converted';
export type ScenarioKind = 'success' | 'error' | 'timeout' | 'custom';
export type RouteStatus = 'active' | 'disabled';
export type ProfileKind = 'static' | 'dynamic';
export type ProxyUrlMode = 'static' | 'prefix';

export interface Project {
  id: string;
  name: string;
  key: string;
  default_proxy_enabled: boolean;
  default_proxy_url: string | null;
  created_at: string;
  updated_at: string;
}

export interface ProjectsResponse {
  items: Project[];
  active_project_id: string;
}

export interface UnknownRequest {
  id: string;
  project_id: string;
  method: string;
  path: string;
  query: Record<string, string>;
  headers: Record<string, string>;
  body_base64: string | null;
  body_text: string | null;
  first_seen_at: string;
  last_seen_at: string;
  count: number;
  status: UnknownRequestStatus;
  converted_route_id: string | null;
}

export interface MockRoute {
  id: string;
  project_id: string;
  method: string;
  path_pattern: string;
  name: string;
  tags: string[];
  status: RouteStatus;
  active_scenario_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface RouteProfile {
  id: string;
  route_id: string;
  name: string;
  profile_kind: ProfileKind;
  kind: ScenarioKind;
  proxy_url: string | null;
  proxy_url_mode: ProxyUrlMode;
  status_code: number;
  response_headers: Record<string, string>;
  response_body: string | null;
  delay_ms: number;
  selection_rules: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface ListResponse<T> {
  items: T[];
}

export interface ConvertForm {
  name: string;
  tags: string;
  scenarioName: string;
  profileKind: ProfileKind;
  kind: ScenarioKind;
  proxyUrl: string;
  proxyUrlMode: ProxyUrlMode;
  statusCode: number;
  responseHeaders: string;
  responseBody: string;
  delayMs: number;
}

export const emptyForm: ConvertForm = {
  name: '',
  tags: '',
  scenarioName: 'success',
  profileKind: 'static',
  kind: 'success',
  proxyUrl: '',
  proxyUrlMode: 'prefix',
  statusCode: 200,
  responseHeaders: '{\n  "content-type": "application/json"\n}',
  responseBody: '{\n  "ok": true\n}',
  delayMs: 0
};

export interface RouteForm {
  method: string;
  pathPattern: string;
  name: string;
  tags: string;
  status: RouteStatus;
}
