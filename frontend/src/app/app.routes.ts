import { Routes } from '@angular/router';

export const routes: Routes = [
  {
    path: '',
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
    loadComponent: () => import('./pages/apps/apps').then((m) => m.Apps),
    title: 'App Store',
  },
  {
    path: 'admin',
    loadComponent: () => import('./pages/admin/admin').then((m) => m.Admin),
    title: 'Admin',
  },
  {
    path: 'my-apps',
    loadComponent: () => import('./pages/my-apps/my-apps').then((m) => m.MyApps),
    title: 'My Apps',
  },
];
