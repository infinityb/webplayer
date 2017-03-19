class Unimplemented {
    //
}

class LoginRequest {
    fap: string;
    faat: string;

    constructor(fap: string, faat: string) {
        this.fap = fap;
        this.faat = faat;
    }

    run(runner: any): Promise<LoginResponse> {
        // raises {AuthError}
        return new Promise(function(resolve, reject) {
            reject(new Unimplemented());
        });
    }
}

class LoginResponse {
    user_id: string;
    access_token: string;
}

class LoginError {
    //
}

class AuthError {
    //
}

class SongSearch {
    q: string;

    constructor(query: string) {
        this.q = query;
    }

    run(runner: any): Promise<SongSearchResponse> {
        // raises {AuthError}
        return new Promise(function(resolve, reject) {
            reject(new Unimplemented());
        });
    }
}

class SongsAllRequest {
    run(runner: any): Promise<SongSearchResponse> {
        // raises {AuthError}
        return new Promise(function(resolve, reject) {
            reject(new Unimplemented());
        });
    }
}

class SongSearchResponse {
    results: Array<Song>;
}

class Song {
    id: number;
    album: Album;
    blob: string;
    length_ms: number;
    track_no: number;
    metadata: Map<string, string>;
}

class Album {
    id: number;
    art_blob: string;
    metadata: Map<string, string>;
}