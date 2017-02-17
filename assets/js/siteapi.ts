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
    album_id: number;
    blob: string;
    length_ms: number;
    track_no: number;
    art_blob: string;
    song_metadata: Map<string, string>;
    album_metadata: Map<string, string>;
}