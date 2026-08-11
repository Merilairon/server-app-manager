import { inject } from '@angular/core';
import { CanActivateFn, Router, Routes } from '@angular/router';
import { catchError, map, of } from 'rxjs';
import { AuthService } from './services/auth.service';

export const authGuard: CanActivateFn = () => {
  const auth = inject(AuthService);
  const router = inject(Router);

  if (auth.isLoggedIn()) {
    return of(true);
  }

  return auth.me().pipe(
    map(() => true),
    catchError(() => of(router.createUrlTree(['/login']))),
  );
};

export const routes: Routes = [
  {
    path: '',
    canActivate: [authGuard],
    loadComponent: () => import('./pages/home/home').then((m) => m.Home),
    title: 'Server App Manager',
  },
  {
    path: 'login',
    loadComponent: () => import('./pages/login/login').then((m) => m.Login),
    title: 'Login',
  },
  {
    path: 'apps',
    canActivate: [authGuard],
    loadComponent: () => import('./pages/apps/apps').then((m) => m.Apps),
    title: 'App Store',
  },
  {
    path: 'admin',
    canActivate: [authGuard],
    loadComponent: () => import('./pages/admin/admin').then((m) => m.Admin),
    title: 'Admin',
  },
  {
    path: 'my-apps',
    canActivate: [authGuard],
    loadComponent: () => import('./pages/my-apps/my-apps').then((m) => m.MyApps),
    title: 'My Apps',
  },
];
