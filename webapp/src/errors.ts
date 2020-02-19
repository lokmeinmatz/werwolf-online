export function getErrorMessage(errId: string) : string {
    if (ErrorMap[errId] != undefined) {
        return `${errId}: ${ErrorMap[errId]}`
    }
    return `Unbekannter Fehler: ${errId}`
}

const ErrorMap = {
    "NoToken": "Hast du vergessen dich zu verbinden? ;)",
    "InvalidSessionID": "Diese Session existiert nicht.",
}