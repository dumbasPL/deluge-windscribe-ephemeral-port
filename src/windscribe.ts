import axios, {AxiosResponse} from 'axios';
import * as qs from 'qs';
import {parse as parseCookie} from 'set-cookie-parser';

// env variables

const username = process.env.WINDSCRIBE_USERNAME;
const password = process.env.WINDSCRIBE_PASSWORD;

if (!username || username.length == 0) {
  console.log('Missing environment variable WINDSCRIBE_USERNAME');
  process.exit(1);
}

if (!password || password.length == 0) {
  console.log('Missing environment variable WINDSCRIBE_PASSWORD');
  process.exit(1);
}

// interfaces

interface CsrfInfo {
  csrfTime: number;
  csrfToken: string;
}

interface PortForwardingInfo {
  epfExpires: number;
  ports: number[];
}

// constants

const userAgent = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.110 Safari/537.36';

// functions

export async function login(): Promise<string> {
  try {
    // get csrf token and time
    const {data: csrfData} = await axios.post<{csrf_token: string, csrf_time: number}>('https://res.windscribe.com/res/logintoken', null, {
      headers: {'User-Agent': userAgent},
    });

    // log in
    const res = await axios.post('https://windscribe.com/login', qs.stringify({
      login: '1',
      upgrade: '0',
      csrf_time: csrfData.csrf_time,
      csrf_token: csrfData.csrf_token,
      username: username,
      password: password,
      code: ''
    }), {
      headers: {'content-type': 'application/x-www-form-urlencoded', 'User-Agent': userAgent},
      maxRedirects: 0,
      validateStatus: status => status == 302,
    });

    // extract the cookie
    return parseCookie(res.headers['set-cookie'], {map: true, decodeValues: true})['ws_session_auth_hash'].value;
  } catch (error) {
    // try to extract windscribe message
    if (error.response) {
      const response = error.response as AxiosResponse<string>;
      const errorMessage = /<div class="content_message error">.*>(.*)<\/div/.exec(response.data);
      if (response.status == 200 && errorMessage && errorMessage[1]) {
        throw new Error(`Failed to log into windscribe: ${errorMessage[1]}`);
      }
    }

    // or throw a generic error if windscribe message not found
    throw new Error(`Failed to log into windscribe: ${error.message}`);
  }
}

export async function getMyAccountCsrfToken(sessionCookie: string): Promise<CsrfInfo> {
  try {
    // get page
    const res = await axios.get<string>('https://windscribe.com/myaccount', {
      headers: {
        'Cookie': `ws_session_auth_hash=${sessionCookie};`,
        'User-Agent': userAgent,
      },
    });

    // extract csrf tokena and time from page content
    const csrfTime = /csrf_time = (\d+);/.exec(res.data)[1];
    const csrfToken = /csrf_token = '(\w+)';/.exec(res.data)[1];

    return {
      csrfTime: +csrfTime,
      csrfToken: csrfToken,
    };
  } catch (error) {
    throw new Error(`Failed to get csrf token from my account page: ${error.message}`);
  }
}

export async function getPortForwardingInfo(sessionCookie: string): Promise<PortForwardingInfo> {
  try {
    // load sub page
    const res = await axios.get<string>('https://windscribe.com/staticips/load', {
      headers: {
        'Cookie': `ws_session_auth_hash=${sessionCookie};`,
        'User-Agent': userAgent,
      }
    });

    // extract data from page
    const epfExpires = res.data.match(/epfExpires = (\d+);/)[1]; // this is always present. set to 0 if no port is active
    const ports = [...res.data.matchAll(/<span>(?<port>\d+)<\/span>/g)].map(x => +x[1]); // this will return an empty array when there are not pots forwarded

    return {
      epfExpires: +epfExpires,
      ports,
    };
  } catch (error) {
    throw new Error(`Failed to get port forwarding info: ${error.message}`);
  }
}

export async function removeEphemeralPort(sessionCookie: string, csrfInfo: CsrfInfo): Promise<void> {
  try {
    // remove port
    const res = await axios.post<{success: number, epf: boolean, message?: string}>('https://windscribe.com/staticips/deleteEphPort', qs.stringify({
      ctime: csrfInfo.csrfTime,
      ctoken: csrfInfo.csrfToken
    }), {
      headers: {
        'content-type': 'application/x-www-form-urlencoded',
        'Cookie': `ws_session_auth_hash=${sessionCookie};`,
        'User-Agent': userAgent,
      }
    });

    // check for errors
    if (res.data.success == 0) {
      throw new Error(`success = 0; ${res.data.message ?? 'No message'}`);
    }

    // make sure we actually removed it
    if (res.data.epf == false) {
      console.log('Tried to remove a non-existent ephemeral port, ignoring');
    } else {
      console.log('Deleted ephemeral port');
    }
  } catch (error) {
    throw new Error(`Failed to delete ephemeral port: ${error.message}`);
  }
}

export async function requestMatchingEphemeralPort(sessionCookie: string, csrfInfo: CsrfInfo): Promise<PortForwardingInfo> {
  try {
    // request new port
    const res = await axios.post<{success: number, message?: string, epf?: {ext: number, int: number, start_ts: number}}>('https://windscribe.com/staticips/postEphPort', qs.stringify({
      ctime: csrfInfo.csrfTime,
      ctoken: csrfInfo.csrfToken,
      port: '', // empty string for a matching port
    }), {
      headers: {
        'content-type': 'application/x-www-form-urlencoded',
        'Cookie': `ws_session_auth_hash=${sessionCookie};`,
        'User-Agent': userAgent,
      }
    });

    // check for errors
    if (res.data.success == 0) {
      throw new Error(`success = 0; ${res.data.message ?? 'No message'}`);
    }

    // epf should be present by this point
    const epf = res.data.epf!;
    console.log(`Created new matching ephemeral port: ${epf.ext}`);
    return {
      epfExpires: epf.start_ts,
      ports: [epf.ext, epf.int],
    };
  } catch (error) {
    throw new Error;
  }
}
