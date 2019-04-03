/* necessary for react transitions to work.
   see https://stackoverflow.com/questions/54698733/types-react-transition-group-generic-type-reactelementp-t-requires-betwee. */
declare module 'react-transition-group' {
    export const CSSTransitionGroup: any
}
