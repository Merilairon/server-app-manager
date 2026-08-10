import { TestBed } from '@angular/core/testing';
import { provideHttpClient } from '@angular/common/http';
import { provideHttpClientTesting, HttpTestingController } from '@angular/common/http/testing';
import { AppsService, App } from './apps.service';

describe('AppsService', () => {
  let service: AppsService;
  let http: HttpTestingController;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        AppsService,
        provideHttpClient(),
        provideHttpClientTesting(),
      ],
    });
    service = TestBed.inject(AppsService);
    http = TestBed.inject(HttpTestingController);
  });

  afterEach(() => {
    http.verify();
  });

  it('should load the catalog', () => {
    const apps: App[] = [
      {
        name: 'Whoami',
        slug: 'whoami',
        image: 'traefik/whoami',
        status: 'available',
        placeholders: [],
      },
    ];
    service.list().subscribe((res) => {
      expect(res.apps).toEqual(apps);
    });

    const req = http.expectOne('/api/v1/apps');
    req.flush({ apps });
  });

  it('should install an app', () => {
    const result = { install_id: 't_whoami', slug: 'whoami', status: 'healthy', url: 'https://whoami.example.com' };
    service.install('whoami', { DOMAIN: 'example.com' }).subscribe((res) => {
      expect(res).toEqual(result);
    });

    const req = http.expectOne('/api/v1/apps/install');
    expect(req.request.method).toBe('POST');
    expect(req.request.body).toEqual({ slug: 'whoami', values: { DOMAIN: 'example.com' } });
    req.flush(result);
  });

  it('should uninstall an app', () => {
    const result = { slug: 'whoami', status: 'uninstalled' };
    service.uninstall('whoami').subscribe((res) => {
      expect(res).toEqual(result);
    });

    const req = http.expectOne('/api/v1/apps/whoami/uninstall');
    expect(req.request.method).toBe('POST');
    req.flush(result);
  });
});
