import { HttpInterceptorFn } from '@angular/common/http';

export const csrfInterceptor: HttpInterceptorFn = (req, next) => {
  if (['POST', 'PUT', 'PATCH', 'DELETE'].includes(req.method)) {
    const token = localStorage.getItem('csrf') ?? '';
    req = req.clone({
      setHeaders: { 'X-CSRF-Token': token },
    });
  }
  return next(req);
};
