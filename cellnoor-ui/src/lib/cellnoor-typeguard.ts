import type { ApiErrorResponse } from "cellnoor-types/ApiErrorResponse";

export function isSuccess<T>(response: T | ApiErrorResponse): response is T {
  return (response as ApiErrorResponse).status === undefined;
}
