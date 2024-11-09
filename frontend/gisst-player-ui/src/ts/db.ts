import * as tus from 'tus-js-client'
import * as SparkMD5 from 'spark-md5'
import {DBRecord, NEVER_UPLOADED_ID} from "./models";

export class GISSTDBConnector {

    repo_url: string;


    constructor(db_url:string) {
        this.repo_url = db_url;
    }

    async getRecordById(record_type: string, record_id: string): Promise<DBRecord> {
        return fetch(
            `${this.repo_url}/${record_type}s/${record_id}`,
            {
                method: 'GET',
                cache: 'no-cache',
                headers: {
                    Accept: 'application/json',
                },
            }
        ).then(response => {
            if(!response.ok){
                throw new Error(response.statusText)
            }
            return response.json() as Promise<DBRecord>
        })
    }

    async uploadRecord(record: DBRecord, record_type: string): Promise<DBRecord> {
        return fetch(
            `${this.repo_url}/${record_type}s/create`,
            {
                method: 'POST',
                cache: 'no-cache',
                headers: {
                    'Content-Type': 'application/json',
                    Accept: 'application/json',
                },
                body: JSON.stringify(record),
            }
        ).then(response => {
            if(!response.ok) {
                throw new Error(response.statusText)
            }
            return response.json() as Promise<DBRecord>
        })
    }

    async uploadFile(file:File, file_id:string,
                     errorCallback: (error:Error) => void,
                     progressCallback: (percentage: number) => void,
                     successCallback: (file_uuid:string) => void
    ) {
        if (file_id != NEVER_UPLOADED_ID) {
          successCallback(file_id);
          return;
        }
        const upload = new tus.Upload(file, {
            endpoint: `${this.repo_url}/resources`,
            retryDelays: [0, 3000, 5000, 10000, 20000],
            chunkSize: 10485760,
            metadata: {
                filename: file.name,
                hash: await computeChecksumMd5(file),
            },
            onError: function (error) {
                console.log('TUS upload failed because: ' + error);
                errorCallback(error);
            },
            onProgress: function (bytesUploaded, bytesTotal) {
                const percentage = ((bytesUploaded / bytesTotal) * 100).toFixed(2)
                console.log(bytesUploaded, bytesTotal, percentage + '%')
                progressCallback(parseFloat(percentage));
            },
            onSuccess: function () {
              console.log('Upload %s to %s', file.name, upload.url)
              const url_parts = upload.url!.split('/');
              const uuid_string = url_parts[url_parts.length - 1];
              successCallback(uuid_string);
            },
        })

        // Check if there are any previous uploads to continue.
        upload.findPreviousUploads().then(function (previousUploads) {
            // Found previous uploads so we select the first one.
            if (previousUploads.length) {
                upload.resumeFromPreviousUpload(previousUploads[0])
            }

            // Start the upload
            upload.start()
        })
    }



}


// code from https://dev.to/qortex/compute-md5-checksum-for-a-file-in-typescript-59a4
function computeChecksumMd5(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
        const chunkSize = 2097152; // Read in chunks of 2MB
        const spark = new SparkMD5.ArrayBuffer();
        const fileReader = new FileReader();

        let cursor = 0; // current cursor in file

        fileReader.onerror = function(): void {
            reject('MD5 computation failed - error reading the file');
        };

        // read chunk starting at `cursor` into memory
        function processChunk(chunk_start: number): void {
            const chunk_end = Math.min(file.size, chunk_start + chunkSize);
            fileReader.readAsArrayBuffer(file.slice(chunk_start, chunk_end));
        }

        // when it's available in memory, process it
        // If using TS >= 3.6, you can use `FileReaderProgressEvent` type instead
        // of `any` for `e` variable, otherwise stick with `any`
        // See https://github.com/Microsoft/TypeScript/issues/25510
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        fileReader.onload = function(e: any): void {
            spark.append(e.target.result); // Accumulate chunk to md5 computation
            cursor += chunkSize; // Move past this chunk

            if (cursor < file.size) {
                // Enqueue next chunk to be accumulated
                processChunk(cursor);
            } else {
                // Computation ended, last chunk has been processed. Return as Promise value.
                // This returns the base64 encoded md5 hash, which is what
                // Rails ActiveStorage or cloud services expect
                // resolve(btoa(spark.end(true)));

                // If you prefer the hexdigest form (looking like
                // '7cf530335b8547945f1a48880bc421b2'), replace the above line with:
                resolve(spark.end());
            }
        };

        processChunk(0);
    });
}
