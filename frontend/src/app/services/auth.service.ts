import { Injectable, computed, inject, signal } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Router } from '@angular/router';
import { tap } from 'rxjs';

export interface User {
  id: string;
  username: string;
  email: string;
  role: string;
}

@Injectable({ providedIn: 'root' })
export class AuthService {
  private readonly http = inject(HttpClient);
  private readonly router = inject(Router);

  private readonly userSignal = signal<User | null>(null);
  readonly user = this.userSignal.asReadonly();
  readonly isLoggedIn = computed(() => !!this.userSignal());
  readonly role = computed(() => this.userSignal()?.role ?? null);

  login(username: string, password: string) {
    return this.http
      .post<{ user: User; csrf: string }>('/api/v1/auth/login', { username, password })
      .pipe(
        tap((res) => {
          localStorage.setItem('csrf', res.csrf);
          this.userSignal.set(res.user);
        }),
      );
  }

  me() {
    return this.http.get<User>('/api/v1/auth/me').pipe(
      tap((user) => {
        this.userSignal.set(user);
      }),
    );
  }

  logout() {
    this.userSignal.set(null);
    localStorage.removeItem('csrf');
    this.router.navigate(['/login']);
  }
}
