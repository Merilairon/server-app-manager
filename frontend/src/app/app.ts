import { Component, inject, signal } from '@angular/core';
import { NavigationEnd, Router, RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';
import { filter } from 'rxjs';
import { AuthService } from './services/auth.service';

@Component({
  selector: 'app-root',
  imports: [RouterOutlet, RouterLink, RouterLinkActive],
  templateUrl: './app.html',
  styleUrl: './app.scss',
})
export class App {
  protected readonly title = signal('Server App Manager');
  private readonly router = inject(Router);
  private readonly auth = inject(AuthService);
  protected readonly isLogin = signal(this.router.url === '/login');

  constructor() {
    this.router.events
      .pipe(filter((e): e is NavigationEnd => e instanceof NavigationEnd))
      .subscribe((e) => {
        this.isLogin.set(e.urlAfterRedirects === '/login');
      });
  }

  protected logout() {
    this.auth.logout();
  }
}
