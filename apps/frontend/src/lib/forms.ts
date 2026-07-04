import type { ConvertForm, UnknownRequest } from '../types';

export function profilePayload(form: ConvertForm) {
  const headers =
    form.profileKind === 'static' ? parseJsonObject(form.responseHeaders, 'Response headers') : {};

  return {
    name: form.scenarioName,
    profile_kind: form.profileKind,
    kind: form.kind,
    proxy_url: form.profileKind === 'dynamic' ? form.proxyUrl : undefined,
    proxy_url_mode: form.profileKind === 'dynamic' ? form.proxyUrlMode : undefined,
    status_code: form.statusCode,
    response_headers: headers,
    response_body: form.profileKind === 'static' ? form.responseBody : undefined,
    delay_ms: form.delayMs,
    selection_rules: {}
  };
}

export function splitTags(value: string): string[] {
  return value
    .split(',')
    .map((tag) => tag.trim())
    .filter(Boolean);
}

export function normalizeHttpMethod(value: string): string {
  return value.trim().toUpperCase();
}

export function projectPath(path: string, projectId: string): string {
  const separator = path.includes('?') ? '&' : '?';
  return `${path}${separator}project_id=${encodeURIComponent(projectId)}`;
}

export function mergeUnknownRequest(requests: UnknownRequest[], next: UnknownRequest): UnknownRequest[] {
  const byId = new Map(requests.map((request) => [request.id, request]));
  byId.set(next.id, next);

  return Array.from(byId.values()).sort(
    (left, right) => new Date(right.last_seen_at).getTime() - new Date(left.last_seen_at).getTime()
  );
}

function socketIoOrigin(apiBaseUrl: string): string {
  if (apiBaseUrl.startsWith('http://') || apiBaseUrl.startsWith('https://')) {
    return new URL(apiBaseUrl).origin;
  }

  return window.location.origin;
}

export function parseJsonObject(value: string, label: string): Record<string, string> {
  const parsed = JSON.parse(value);
  if (!parsed || Array.isArray(parsed) || typeof parsed !== 'object') {
    throw new Error(`${label} must be a JSON object`);
  }

  return parsed;
}

export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : 'Request failed';
}

export function formatDate(value: string): string {
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short'
  }).format(new Date(value));
}
