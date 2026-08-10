import { TestBed } from '@angular/core/testing';
import { provideHttpClient } from '@angular/common/http';
import { provideHttpClientTesting, HttpTestingController } from '@angular/common/http/testing';
import { Router } from '@angular/router';
import { AuthService, User } from './auth.service';

const routerMock = {
  navigate: () => Promise.resolve(true),
};

describe('AuthService', () => {
  let service: AuthService;
  let http: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        AuthService,
        provideHttpClient(),
        provideHttpClientTesting(),
        { provide: Router, useValue: routerMock },
      ],
    });
    service = TestBed.inject(AuthService);
    http = TestBed.inject(HttpTestingController);
    localStorage.clear();
  });

  afterEach(() => {
    http.verify();
    localStorage.clear();
  });

  it('should store csrf and user after login', () => {
    const user: User = {
      id: '1',
      username: 'admin',
      email: 'admin@example.com',
      role: 'admin',
    };
    service.login('admin', 'secret').subscribe((res) => {
      expect(res.user).toEqual(user);
      expect(res.csrf).toBe('token-123');
      expect(localStorage.getItem('csrf')).toBe('token-123');
      expect(service.user()).toEqual(user);
      expect(service.isLoggedIn()).toBe(true);
    });

    const req = http.expectOne('/api/v1/auth/login');
    req.flush({ user, csrf: 'token-123' });
  });

  it('should clear user and csrf on logout', () => {
    localStorage.setItem('csrf', 'token-123');
    service.logout();
    expect(service.user()).toBeNull();
    expect(localStorage.getItem('csrf')).toBeNull();
  });
});
