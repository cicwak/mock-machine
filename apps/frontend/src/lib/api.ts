import { API_BASE_URL } from '../config';

export async function apiGet<T>(path: string): Promise<T> {
  return apiRequest<T>(path);
}

export async function apiPost<T>(path: string, body: unknown): Promise<T> {
  return apiRequest<T>(path, 'POST', body);
}

export async function apiPut<T>(path: string, body: unknown): Promise<T> {
  return apiRequest<T>(path, 'PUT', body);
}

async function apiRequest<T>(path: string, method = 'GET', body?: unknown): Promise<T> {
  const response = await fetch(`${API_BASE_URL}${path}`, {
    method,
    ...(body === undefined
      ? {}
      : {
          headers: {
            'content-type': 'application/json'
          },
          body: JSON.stringify(body)
        })
  });

  return parseResponse<T>(response);
}

async function parseResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    const payload = await response.json().catch(() => null);
    throw new Error(payload?.error ?? `${response.status} ${response.statusText}`);
  }

  return response.json() as Promise<T>;
}
