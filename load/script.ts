import http from 'k6/http';
import { sleep } from 'k6';

export const options = {
  stages: [
    { duration: '30s', target: 100 },
    { duration: '2m', target: 100 },
    { duration: '10s', target: 500 },
    { duration: '5m', target: 500 },
    { duration: '20s', target: 200 },
    { duration: '1m', target: 200 },
    { duration: '1m', target: 0 },
  ],
};

export default function () {
  http.get(`http://${__ENV.SERVER_URL ?? 'localhost:8080'}/api/v1/units`);
  sleep(1);
}
