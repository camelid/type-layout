-- The type equivalent to Maybe Bool
-- <None of {} | Some of <False of {} | True of {}>>

-- The None value (aka, Nothing)
<None = {}> as <None of {} | Some of <False of {} | True of {}>>

-- The Some False value (aka, Just False)
<Some = <False = {}> as <False of {} | True of {}>> as <None of {} | Some of <False of {} | True of {}>>

-- The Some True value
<Some = <True = {}> as <False of {} | True of {}>> as <None of {} | Some of <False of {} | True of {}>>


-- A basic pattern match. Equivalent to `opt.unwrap_or(false)` in Rust.
alias Bool = <False of {} | True of {}> in
-- Maybe Bool
alias MBool = <None of {} | Some of Bool> in
let Some_of_True : MBool = <Some = <True = {}> as Bool> as MBool  in
let Some_of_False : MBool = <Some = <False = {}> as Bool> as MBool  in
let None : MBool = <None = {}> as MBool  in
match None : MBool {
    <None = _ : {}> as MBool => <False = {}> as Bool,
    <Some = b : Bool> as MBool => b : Bool,
}
