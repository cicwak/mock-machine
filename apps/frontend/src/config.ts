import type { RouteStatus, UnknownRequestStatus } from './types';

export const API_BASE_URL =
  import.meta.env.VITE_API_BASE_URL ??
  (import.meta.env.DEV ? 'http://127.0.0.1:8080/mockadminapi' : '/mockadminapi');

export const SOCKET_IO_URL = import.meta.env.VITE_SOCKET_IO_URL ?? socketIoOrigin(API_BASE_URL);
export const UNKNOWN_REQUEST_CAPTURED_EVENT = 'unknown_request:captured';

export const HTTP_METHODS = [
  'ACL',
  'BASELINE-CONTROL',
  'BIND',
  'CHECKIN',
  'CHECKOUT',
  'CONNECT',
  'COPY',
  'DELETE',
  'GET',
  'HEAD',
  'LABEL',
  'LINK',
  'LOCK',
  'MERGE',
  'MKACTIVITY',
  'MKCALENDAR',
  'MKCOL',
  'MKREDIRECTREF',
  'MKWORKSPACE',
  'MOVE',
  'OPTIONS',
  'ORDERPATCH',
  'PATCH',
  'POST',
  'PRI',
  'PROPFIND',
  'PROPPATCH',
  'PUT',
  'QUERY',
  'REBIND',
  'REPORT',
  'SEARCH',
  'TRACE',
  'UNBIND',
  'UNCHECKOUT',
  'UNLINK',
  'UNLOCK',
  'UPDATE',
  'UPDATEREDIRECTREF',
  'VERSION-CONTROL'
] as const;

export const COMMON_HTTP_METHODS = [
  'CONNECT',
  'DELETE',
  'GET',
  'HEAD',
  'OPTIONS',
  'PATCH',
  'POST',
  'PUT',
  'QUERY',
  'TRACE'
] as const;

type CommonHttpMethod = (typeof COMMON_HTTP_METHODS)[number];

export const HTTP_METHOD_OPTIONS = [
  ...COMMON_HTTP_METHODS.map((method) => ({ method, group: 'Common' })),
  ...HTTP_METHODS.filter((method) => !COMMON_HTTP_METHODS.includes(method as CommonHttpMethod)).map(
    (method) => ({ method, group: 'Other' })
  )
];

export const unknownStatusColors: Record<UnknownRequestStatus, 'default' | 'success' | 'warning'> = {
  new: 'warning',
  ignored: 'default',
  converted: 'success'
};

export const routeStatusColors: Record<RouteStatus, 'default' | 'success'> = {
  active: 'success',
  disabled: 'default'
};

function socketIoOrigin(apiBaseUrl: string): string {
  if (apiBaseUrl.startsWith('http://') || apiBaseUrl.startsWith('https://')) {
    return new URL(apiBaseUrl).origin;
  }

  return window.location.origin;
}
